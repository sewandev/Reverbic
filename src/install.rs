#[cfg(target_os = "windows")]
pub fn maybe_self_install() {
    let Some(current) = std::env::current_exe().ok() else {
        return;
    };
    let Some(local_appdata) = std::env::var("LOCALAPPDATA").ok() else {
        return;
    };

    let install_dir = std::path::PathBuf::from(&local_appdata)
        .join("Programs")
        .join("reverbic");

    if current.starts_with(&install_dir) {
        return;
    }

    if install_dir_in_user_path(&install_dir) {
        return;
    }

    if std::fs::create_dir_all(&install_dir).is_err() {
        return;
    }

    let dest = install_dir.join("reverbic.exe");
    if std::fs::copy(&current, &dest).is_err() {
        return;
    }

    add_to_user_path(&install_dir);

    println!("reverbic installed to {}", dest.display());
    println!("Open a new terminal and type: reverbic");
}

#[cfg(not(target_os = "windows"))]
pub fn maybe_self_install() {}

#[cfg(target_os = "windows")]
fn install_dir_in_user_path(dir: &std::path::Path) -> bool {
    let dir_lower = dir.to_string_lossy().to_lowercase();
    let Ok(out) = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[Environment]::GetEnvironmentVariable('PATH', 'User')",
        ])
        .output()
    else {
        return false;
    };
    String::from_utf8_lossy(&out.stdout)
        .to_lowercase()
        .contains(dir_lower.as_str())
}

#[cfg(target_os = "windows")]
fn add_to_user_path(dir: &std::path::Path) {
    let script = "$p = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
                  $dir = $env:REVERBIC_INSTALL_DIR; \
                  [Environment]::SetEnvironmentVariable('PATH', \"$p;$dir\", 'User')";
    let _ = std::process::Command::new("powershell")
        .env("REVERBIC_INSTALL_DIR", dir)
        .args(["-NoProfile", "-Command", script])
        .output();
}
