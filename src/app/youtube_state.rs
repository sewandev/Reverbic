use crate::integrations::youtube::{ResolvedYoutubePlayback, YoutubeError, YoutubeVideo};

type InstallRx = std::sync::mpsc::Receiver<Result<std::path::PathBuf, YoutubeError>>;
type SearchRx = std::sync::mpsc::Receiver<Result<Vec<YoutubeVideo>, YoutubeError>>;
type ResolveRx = std::sync::mpsc::Receiver<Result<ResolvedYoutubePlayback, YoutubeError>>;

pub enum YoutubeStatus {
    Idle,
    Installing,
    Ready,
    Resolving,
    Error(String),
}

pub struct YoutubeState {
    pub status: YoutubeStatus,
    pub query: String,
    pub results: Vec<YoutubeVideo>,
    pub loading: bool,
    pub selected: usize,
    pub scroll_offset: usize,
    pub(super) search_pending_until: Option<std::time::Instant>,
    pub(super) install_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) install_rx: Option<InstallRx>,
    pub(super) search_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) search_rx: Option<SearchRx>,
    pub(super) resolve_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) resolve_rx: Option<ResolveRx>,
}

impl YoutubeState {
    pub fn cleanup(&mut self) {
        fn abort(handle: &mut Option<tokio::task::JoinHandle<()>>) {
            if let Some(task) = handle.take() {
                task.abort();
            }
        }

        abort(&mut self.install_task);
        abort(&mut self.search_task);
        abort(&mut self.resolve_task);
        self.search_pending_until = None;
    }
}

impl Default for YoutubeState {
    fn default() -> Self {
        Self {
            status: YoutubeStatus::Idle,
            query: String::new(),
            results: Vec::new(),
            loading: false,
            selected: 0,
            scroll_offset: 0,
            search_pending_until: None,
            install_task: None,
            install_rx: None,
            search_task: None,
            search_rx: None,
            resolve_task: None,
            resolve_rx: None,
        }
    }
}
