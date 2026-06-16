use super::modal::YoutubeSubTab;
use crate::integrations::youtube::{
    ResolvedYoutubePlayback, YoutubeChapter, YoutubeError, YoutubePlaylist, YoutubeVideo,
};

type InstallRx = std::sync::mpsc::Receiver<Result<std::path::PathBuf, YoutubeError>>;
type VideosRx = std::sync::mpsc::Receiver<Result<Vec<YoutubeVideo>, YoutubeError>>;
type PlaylistsRx = std::sync::mpsc::Receiver<Result<Vec<YoutubePlaylist>, YoutubeError>>;
type ResolveRx = std::sync::mpsc::Receiver<Result<ResolvedYoutubePlayback, YoutubeError>>;
type SegmentsRx = std::sync::mpsc::Receiver<Result<Vec<(f32, f32)>, String>>;

pub enum YoutubeStatus {
    Idle,
    Installing,
    Ready,
    Resolving,
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum YoutubePlaybackContext {
    SearchResults,
    Bookmarks,
    LikedVideos,
    PlaylistVideos,
    Mix,
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

    pub public_query: String,
    pub public_results: Vec<YoutubePlaylist>,
    pub public_selected: usize,
    pub public_scroll_offset: usize,
    pub public_loading: bool,
    pub(super) public_search_pending_until: Option<std::time::Instant>,
    pub(super) public_search_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) public_search_rx: Option<PlaylistsRx>,

    pub bookmarks: Vec<YoutubeVideo>,
    pub bookmarks_selected: usize,
    pub bookmarks_scroll_offset: usize,

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

    pub(super) validate_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) validate_rx: Option<VideosRx>,
    pub(super) validate_silent: bool,
    pub session_health: Option<bool>,
    pub cookies_invalid: bool,

    pub(super) preresolve_last_id: Option<String>,
    pub(super) preresolve_deadline: Option<std::time::Instant>,
    pub(super) preresolve_video: Option<YoutubeVideo>,
    pub(super) preresolve_task: Option<tokio::task::JoinHandle<()>>,

    pub mix_videos: Vec<YoutubeVideo>,
    pub mix_loading: bool,
    pub(super) mix_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) mix_rx: Option<VideosRx>,
    pub(super) mix_is_extension: bool,
    pub(super) mix_resume_on_extend: bool,
    pub(super) mix_seed_to_skip: Option<String>,

    pub(super) sb_segments: Vec<(f32, f32)>,
    pub(super) sb_station_key: Option<String>,
    pub(super) sb_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) sb_rx: Option<SegmentsRx>,
    pub(super) sb_cooldown_until: Option<std::time::Instant>,

    pub(super) playing_chapters: Vec<YoutubeChapter>,
    pub(super) chapter_station_key: Option<String>,
    pub(super) chapter_video_title: String,
    pub(super) chapter_video_channel: String,
    pub(super) current_chapter: Option<usize>,

    pub playback_context: Option<(YoutubePlaybackContext, usize)>,
    pub was_playing: bool,
    pub crossfade_from: Option<String>,
}

impl YoutubeState {
    pub fn validating(&self) -> bool {
        self.validate_task.is_some()
    }

    pub fn cleanup(&mut self) {
        fn abort(handle: &mut Option<tokio::task::JoinHandle<()>>) {
            if let Some(task) = handle.take() {
                task.abort();
            }
        }

        abort(&mut self.install_task);
        abort(&mut self.search_task);
        abort(&mut self.public_search_task);
        abort(&mut self.resolve_task);
        abort(&mut self.liked_task);
        abort(&mut self.playlists_task);
        abort(&mut self.playlist_videos_task);
        abort(&mut self.validate_task);
        self.validate_silent = false;
        abort(&mut self.preresolve_task);
        abort(&mut self.mix_task);
        abort(&mut self.sb_task);
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
            public_query: String::new(),
            public_results: Vec::new(),
            public_selected: 0,
            public_scroll_offset: 0,
            public_loading: false,
            public_search_pending_until: None,
            public_search_task: None,
            public_search_rx: None,
            bookmarks: Vec::new(),
            bookmarks_selected: 0,
            bookmarks_scroll_offset: 0,
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
            validate_task: None,
            validate_rx: None,
            validate_silent: false,
            session_health: None,
            cookies_invalid: false,
            preresolve_last_id: None,
            preresolve_deadline: None,
            preresolve_video: None,
            preresolve_task: None,
            mix_videos: Vec::new(),
            mix_loading: false,
            mix_task: None,
            mix_rx: None,
            mix_is_extension: false,
            mix_resume_on_extend: false,
            mix_seed_to_skip: None,
            sb_segments: Vec::new(),
            sb_station_key: None,
            sb_task: None,
            sb_rx: None,
            sb_cooldown_until: None,
            playing_chapters: Vec::new(),
            chapter_station_key: None,
            chapter_video_title: String::new(),
            chapter_video_channel: String::new(),
            current_chapter: None,
            playback_context: None,
            was_playing: false,
            crossfade_from: None,
        }
    }
}
