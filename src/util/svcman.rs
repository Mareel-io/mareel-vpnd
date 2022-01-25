use super::super::svc;

pub(crate) fn svc_install(method: &str, config: &Option<String>) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::install(config).unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

pub(crate) fn svc_uninstall(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::uninstall().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

pub(crate) fn svc_start(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::start().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}

pub(crate) fn svc_stop(method: &str) -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        match method {
            "systemd" => svc::systemd::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        match method {
            "winsvc" => svc::winsvc::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        match method {
            "launchd" => svc::launchd::stop().unwrap(),
            _ => panic!("Not supported feature: {}", method),
        };
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
        panic!("Not supported yet!");
    }
}
