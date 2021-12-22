use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const LAUNCHD_UNIT_PATH: &str = "/Library/LaunchDaemons/io.mareel.vpnd.plist";

pub fn install(config: &Option<String>) -> Result<(), ()> {
    let service_binary_path = ::std::env::current_exe().unwrap();
    let mut working_dir = ::std::env::current_exe().unwrap();
    working_dir.pop();
    // ugly xml...
    let exec_cmd = match config {
        Some(x) => format!(
            "<string>{}</string><string>--config</string><string>{}</string>",
            service_binary_path.to_string_lossy(),
            std::borrow::Cow::Borrowed(x),
        ),
        None => format!("<string>{}</string>", service_binary_path.to_string_lossy()),
    };
    let launchd_unit_path: PathBuf = LAUNCHD_UNIT_PATH.into();
    let launchd_unit = format!(
        r##"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
        <key>StandardErrorPath</key>
        <string>/var/log/mareel-vpnd.log</string>
        <key>StandardOutPath</key>
        <string>/var/log/mareel-vpnd.log</string>
        <key>UserName</key>
        <string>root</string>
        <key>KeepAlive</key>
        <true/>
        <key>Label</key>
        <string>io.mareel.vpnd</string>
        <key>RunAtLoad</key>
        <true/>
        <key>SoftResourceLimits</key>
        <dict>
                <key>NumberOfFiles</key>
                <integer>1024</integer>
        </dict>
        <key>WorkingDirectory</key>
        <string>{}</string>
        <key>ProgramArguments</key>
        <array>
            {}
        </array>
</dict>
</plist>
"##,
        working_dir.to_str().unwrap(),
        exec_cmd
    );

    println!(
        "Installed unit with workdir {}",
        working_dir.to_str().unwrap()
    );

    let mut unit_file = File::create(launchd_unit_path).unwrap();

    unit_file.write(launchd_unit.as_bytes()).unwrap();
    unit_file.sync_all().unwrap();
    drop(unit_file);

    Ok(())
}

pub fn start() -> Result<(), ()> {
    Command::new("launchctl")
        .arg("load")
        .arg(LAUNCHD_UNIT_PATH)
        .output()
        .expect("Failed to start service!");
    Ok(())
}

pub fn stop() -> Result<(), ()> {
    Command::new("launchctl")
        .arg("unload")
        .arg(LAUNCHD_UNIT_PATH)
        .output()
        .expect("Failed to stop service!");
    Ok(())
}

pub fn uninstall() -> Result<(), ()> {
    Command::new("launchctl")
        .arg("unload")
        .arg(LAUNCHD_UNIT_PATH)
        .output()
        .expect("Failed to stop service!");

    remove_file(LAUNCHD_UNIT_PATH).expect("Failed to remove service!");
    Ok(())
}
