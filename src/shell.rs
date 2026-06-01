pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::{Command, Stdio};
        let result = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Start-Process \"{}\"", url.replace('"', "`\"")),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Err(e) = result {
            tracing::error!("open_url: no se pudo abrir el navegador: {e}");
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
            tracing::error!("open_url: no se pudo abrir el navegador: {e}");
        }
    }
}
