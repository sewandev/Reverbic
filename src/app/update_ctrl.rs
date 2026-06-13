use std::path::PathBuf;
use std::sync::mpsc;

use super::App;
use crate::update::UpdateAsset;

impl App {
    pub fn start_update_check(&mut self) {
        if !self.config.auto_update {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.update_check_rx = Some(rx);
        self.update_check_task = Some(tokio::spawn(async move {
            let result = crate::update::fetch_latest_update().await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_update_check(&mut self) {
        let Some(rx) = &self.update_check_rx else {
            return;
        };
        if let Ok(result) = rx.try_recv() {
            self.update_check_rx = None;
            self.update_check_task = None;
            if let Some(asset) = result {
                self.update_available = Some(asset.version.clone());
                crate::terminal::set_title(&format!(
                    "Reverbic - Downloading v{}...",
                    asset.version
                ));
                self.start_update_download(asset);
            } else {
                tracing::debug!("No compatible update available");
                self.update_available = None;
                self.update_path = None;
                crate::terminal::set_title(concat!("Reverbic v", env!("CARGO_PKG_VERSION")));
            }
        }
    }

    fn start_update_download(&mut self, asset: UpdateAsset) {
        let (tx, rx): (mpsc::SyncSender<Option<PathBuf>>, _) = mpsc::sync_channel(1);
        self.update_download_rx = Some(rx);
        self.update_download_task = Some(tokio::spawn(async move {
            let result = crate::update::download_update(&asset).await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_update_download(&mut self) {
        let Some(rx) = &self.update_download_rx else {
            return;
        };
        if let Ok(result) = rx.try_recv() {
            self.update_download_rx = None;
            self.update_download_task = None;
            if let Some(path) = result {
                self.update_path = Some(path);
                if let Some(version) = &self.update_available {
                    crate::terminal::set_title(&format!("Reverbic - Update v{} Ready", version));
                }
            } else {
                tracing::debug!("Update download failed or was rejected");
                self.update_available = None;
                self.update_path = None;
                crate::terminal::set_title(concat!("Reverbic v", env!("CARGO_PKG_VERSION")));
            }
        }
    }
}
