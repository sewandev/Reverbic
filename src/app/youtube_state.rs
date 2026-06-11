use super::modal::YoutubeSubTab;
use crate::integrations::youtube::{
    ResolvedYoutubePlayback, YoutubeError, YoutubePlaylist, YoutubeVideo,
};

type InstallRx = std::sync::mpsc::Receiver<Result<std::path::PathBuf, YoutubeError>>;
type VideosRx = std::sync::mpsc::Receiver<Result<Vec<YoutubeVideo>, YoutubeError>>;
type PlaylistsRx = std::sync::mpsc::Receiver<Result<Vec<YoutubePlaylist>, YoutubeError>>;
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
    pub(super) search_rx: Option<VideosRx>,
    pub(super) resolve_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) resolve_rx: Option<ResolveRx>,

    pub sub_tab: YoutubeSubTab,

    pub liked_videos: Vec<YoutubeVideo>,
    pub liked_selected: usize,
    pub liked_scroll_offset: usize,
    pub liked_loading: bool,
    pub(super) liked_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) liked_rx: Option<VideosRx>,

    pub playlists: Vec<YoutubePlaylist>,
    pub playlists_selected: usize,
    pub playlists_scroll_offset: usize,
    pub playlists_loading: bool,
    pub(super) playlists_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playlists_rx: Option<PlaylistsRx>,

    pub open_playlist: Option<YoutubePlaylist>,
    pub playlist_videos: Vec<YoutubeVideo>,
    pub playlist_videos_selected: usize,
    pub playlist_videos_scroll_offset: usize,
    pub playlist_videos_loading: bool,
    pub(super) playlist_videos_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playlist_videos_rx: Option<VideosRx>,
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
        abort(&mut self.liked_task);
        abort(&mut self.playlists_task);
        abort(&mut self.playlist_videos_task);
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
            sub_tab: YoutubeSubTab::default(),
            liked_videos: Vec::new(),
            liked_selected: 0,
            liked_scroll_offset: 0,
            liked_loading: false,
            liked_task: None,
            liked_rx: None,
            playlists: Vec::new(),
            playlists_selected: 0,
            playlists_scroll_offset: 0,
            playlists_loading: false,
            playlists_task: None,
            playlists_rx: None,
            open_playlist: None,
            playlist_videos: Vec::new(),
            playlist_videos_selected: 0,
            playlist_videos_scroll_offset: 0,
            playlist_videos_loading: false,
            playlist_videos_task: None,
            playlist_videos_rx: None,
        }
    }
}
