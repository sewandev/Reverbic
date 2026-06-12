pub fn open_url(url: &str) {
    use std::process::Stdio;

    let mut command = platform_open_command(url);
    let result = command.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    if let Err(e) = result {
        tracing::error!("open_url: failed to open browser: {e}");
    }
}

pub fn open_folder(path: &std::path::Path) {
    use std::process::Stdio;

    let mut command = platform_open_folder_command(path);
    let result = command.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    if let Err(e) = result {
        tracing::error!("open_folder: failed to open file explorer: {e}");
    }
}

#[cfg(target_os = "windows")]
fn platform_open_folder_command(path: &std::path::Path) -> std::process::Command {
    let mut command = std::process::Command::new("explorer.exe");
    command.arg(path);
    command
}

#[cfg(target_os = "macos")]
fn platform_open_folder_command(path: &std::path::Path) -> std::process::Command {
    let mut command = std::process::Command::new("open");
    command.arg(path);
    command
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn platform_open_folder_command(path: &std::path::Path) -> std::process::Command {
    let mut command = std::process::Command::new("xdg-open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn platform_open_command(url: &str) -> std::process::Command {
    let mut command = std::process::Command::new("rundll32.exe");
    command.args(["url.dll,FileProtocolHandler", url]);
    command
}

#[cfg(target_os = "macos")]
fn platform_open_command(url: &str) -> std::process::Command {
    let mut command = std::process::Command::new("open");
    command.arg(url);
    command
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn platform_open_command(url: &str) -> std::process::Command {
    let mut command = std::process::Command::new("xdg-open");
    command.arg(url);
    command
}
