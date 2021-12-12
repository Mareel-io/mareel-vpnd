// Most of code come from windows_service crate example
// Licensed under Apache-2.0

use rocket::futures::executor::block_on;
use std::ffi::{OsStr, OsString};
use std::thread;
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

use std::time::Duration;
use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
    ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::{define_windows_service, service_dispatcher};

use rocket::tokio::sync::mpsc;

#[cfg(target_os = "windows")]
extern crate windows_service;

const SERVICE_ID: &str = "mareel_vpn";
const SERVICE_NAME: &str = "Mareel VPN";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const SERVICE_DESC: &str = "Mareel VPN daemon";

pub fn run() -> windows_service::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

define_windows_service!(ffi_service_main, service_main);

pub fn service_main(_args: Vec<OsString>) {
    if let Err(_e) = run_service() {
        //
    }
}

pub fn run_service() -> windows_service::Result<()> {
    let (shdn_tx, shdn_rx) = mpsc::channel::<()>(5);

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                block_on(shdn_tx.send(())).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    super::super::launcher(Some(shdn_rx)).unwrap();

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

pub fn install(config: &Option<String>) -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // This example installs the service defined in `examples/ping_service.rs`.
    // In the real world code you would set the executable path to point to your own binary
    // that implements windows service.
    let service_binary_path = ::std::env::current_exe().unwrap();

    let args: Vec<OsString> = match config {
        Some(x) => vec!["--config".into(), x.into()],
        None => vec![],
    };

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_ID),
        display_name: OsString::from(SERVICE_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: args,
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESC)?;
    Ok(())
}

pub fn start() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    let service = service_manager.open_service(SERVICE_ID, ServiceAccess::START)?;
    service.start(&[OsStr::new("Started using CLI")])?;
    Ok(())
}

pub fn stop() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    let service = service_manager.open_service(SERVICE_ID, ServiceAccess::STOP)?;
    service.stop()?;
    Ok(())
}

pub fn uninstall() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_ID, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    service.delete()?;
    Ok(())
}
