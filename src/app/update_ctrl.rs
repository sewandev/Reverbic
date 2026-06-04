
use std::path::PathBuf;
use std::sync::mpsc;

use super::App;

impl App {
    pub fn start_update_check(&mut self) {
        if !self.config.auto_update { return; }
        let (tx, rx) = mpsc::channel();
        self.update_check_rx = Some(rx);
        self.update_check_task = Some(tokio::spawn(async move {
            let result = crate::update::fetch_latest_version().await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_update_check(&mut self) {
        let Some(rx) = &self.update_check_rx else { return };
        if let Ok(result) = rx.try_recv() {
            self.update_check_rx = None;
            self.update_check_task = None;
            if let Some(version) = result {
                self.update_available = Some(version.clone());
                self.start_update_download(version);
            }
        }
    }

    fn start_update_download(&mut self, version: String) {
        let (tx, rx): (mpsc::SyncSender<PathBuf>, _) = mpsc::sync_channel(1);
        self.update_download_rx = Some(rx);
        self.update_download_task = Some(tokio::spawn(async move {
            if let Some(path) = crate::update::download_update(&version).await {
                let _ = tx.send(path);
            }
        }));
    }

    pub fn poll_update_download(&mut self) {
        let Some(rx) = &self.update_download_rx else { return };
        if let Ok(path) = rx.try_recv() {
            self.update_download_rx = None;
            self.update_download_task = None;
            self.update_path = Some(path);
        }
    }
}
