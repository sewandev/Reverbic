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

    let is_installer = std::env::var("REVERBIC_INSTALLER").as_deref() == Ok("1");
    let in_path = install_dir_in_user_path(&install_dir);

    if in_path && !is_installer {
        return;
    }

    if std::fs::create_dir_all(&install_dir).is_err() {
        return;
    }

    let dest = install_dir.join("reverbic.exe");

    if dest.exists() && is_installer {
        if let (Ok(current_hash), Ok(dest_hash)) = (sha256_file(&current), sha256_file(&dest)) {
            if current_hash == dest_hash {
                if !in_path {
                    add_to_user_path(&install_dir);
                }
                println!("reverbic installed to {}", dest.display());
                println!("Open a new terminal and type: reverbic");
                return;
            }
        }
    }

    if std::fs::copy(&current, &dest).is_err() {
        let old = install_dir.join("reverbic.exe.old");
        let _ = std::fs::rename(&dest, &old);
        if std::fs::copy(&current, &dest).is_err() {
            return;
        }
    }

    if !in_path {
        add_to_user_path(&install_dir);
    }

    println!("reverbic installed to {}", dest.display());
    println!("Open a new terminal and type: reverbic");
}

#[cfg(target_os = "windows")]
fn sha256_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 16 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        }))
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
