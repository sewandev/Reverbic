pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::{Command, Stdio};
        let result = Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Err(e) = result {
            tracing::error!("open_url: failed to open browser: {e}");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::{Command, Stdio};
        let result = Command::new("xdg-open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Err(e) = result {
            tracing::error!("open_url: failed to open browser: {e}");
        }
    }
}
