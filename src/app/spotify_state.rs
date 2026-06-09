use std::collections::VecDeque;

use super::modal::{SpotifyAuthStatus, SpotifyPlayerStatus, SpotifySubTab};
use crate::integrations::spotify::{
    devices::SpotifyDevice, player::SpotifyPlayerHandle, playlists::SpotifyPlaylist, AuthResult,
    SpotifyAlbum, SpotifyError, SpotifyPlaybackState, SpotifyPlayerEvent, SpotifyTrack,
};

pub(super) struct SpotifySearchPage {
    pub(super) generation: u64,
    pub(super) query: String,
    pub(super) offset: usize,
    pub(super) results: Vec<SpotifyTrack>,
    pub(super) has_more: bool,
    pub(super) rate_limit_secs: Option<u64>,
}

type SearchPageRx = std::sync::mpsc::Receiver<SpotifySearchPage>;
type TracksResultRx = std::sync::mpsc::Receiver<Result<(Vec<SpotifyTrack>, bool), SpotifyError>>;
type PlaylistsResultRx =
    std::sync::mpsc::Receiver<Result<(Vec<SpotifyPlaylist>, bool), SpotifyError>>;
type AlbumsResultRx = std::sync::mpsc::Receiver<Result<(Vec<SpotifyAlbum>, bool), SpotifyError>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpotifyPlaybackBackend {
    Remote,
    Native,
}

pub struct SpotifyState {
    pub status: SpotifyAuthStatus,
    pub is_premium: bool,
    pub access_token: Option<String>,
    pub now_playing: Option<SpotifyTrack>,
    pub player_status: SpotifyPlayerStatus,
    pub active_device_id: Option<String>,
    pub native_available: bool,
    pub native_error: Option<String>,

    pub sub_tab: SpotifySubTab,

    pub search_query: String,
    pub search_results: Vec<SpotifyTrack>,
    pub search_loading: bool,
    pub search_loading_more: bool,
    pub search_selected: usize,
    pub search_scroll_offset: usize,
    pub search_offset: usize,
    pub search_has_more: bool,
    pub search_rate_limited: bool,
    pub(super) search_generation: u64,
    pub rate_limited_until: Option<std::time::Instant>,
    pub volume_pending_until: Option<std::time::Instant>,

    pub devices: Vec<SpotifyDevice>,
    pub devices_loading: bool,

    pub playback: Option<SpotifyPlaybackState>,
    pub(super) active_backend: Option<SpotifyPlaybackBackend>,

    pub(super) player_tx: Option<SpotifyPlayerHandle>,
    pub(super) player_rx: Option<std::sync::mpsc::Receiver<SpotifyPlayerEvent>>,
    pub(super) auth_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) auth_rx: Option<std::sync::mpsc::Receiver<AuthResult>>,
    pub(super) search_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) search_rx: Option<SearchPageRx>,
    pub(super) search_more_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) search_more_rx: Option<SearchPageRx>,
    pub(super) devices_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) devices_rx:
        Option<std::sync::mpsc::Receiver<Result<Vec<SpotifyDevice>, SpotifyError>>>,
    pub(super) playback_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playback_rx: Option<std::sync::mpsc::Receiver<Option<SpotifyPlaybackState>>>,

    pub(super) token_refreshed_at: Option<std::time::Instant>,
    pub(super) token_refresh_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) token_refresh_rx:
        Option<std::sync::mpsc::Receiver<Result<(String, String), String>>>,

    pub(super) play_result_rx: Option<std::sync::mpsc::Receiver<Result<(), SpotifyError>>>,
    pub(super) save_track_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,

    pub playback_queue: VecDeque<SpotifyTrack>,
    pub radio_queue: VecDeque<SpotifyTrack>,
    pub recently_played: VecDeque<String>,
    pub(super) radio_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) radio_rx: Option<std::sync::mpsc::Receiver<Vec<SpotifyTrack>>>,

    pub liked_tracks: Vec<SpotifyTrack>,
    pub liked_selected: usize,
    pub liked_scroll_offset: usize,
    pub liked_loading: bool,
    pub liked_has_more: bool,
    pub liked_offset: usize,
    pub liked_rate_limited_until: Option<std::time::Instant>,
    pub(super) liked_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) liked_rx: Option<TracksResultRx>,

    pub playlists: Vec<SpotifyPlaylist>,
    pub playlists_selected: usize,
    pub playlists_scroll_offset: usize,
    pub playlists_loading: bool,
    pub playlists_has_more: bool,
    pub playlists_offset: usize,
    pub open_playlist: Option<SpotifyPlaylist>,
    pub playlist_tracks: Vec<SpotifyTrack>,
    pub playlist_tracks_selected: usize,
    pub playlist_tracks_scroll_offset: usize,
    pub playlist_tracks_loading: bool,
    pub playlist_tracks_has_more: bool,
    pub playlist_tracks_offset: usize,
    pub(super) playlists_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playlists_rx: Option<PlaylistsResultRx>,
    pub(super) playlist_tracks_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playlist_tracks_rx: Option<TracksResultRx>,

    pub top_tracks: Vec<SpotifyTrack>,
    pub top_tracks_selected: usize,
    pub top_tracks_scroll_offset: usize,
    pub top_tracks_loading: bool,
    pub(super) top_tracks_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) top_tracks_rx:
        Option<std::sync::mpsc::Receiver<Result<Vec<SpotifyTrack>, SpotifyError>>>,

    pub recent_tracks: Vec<SpotifyTrack>,
    pub recent_tracks_selected: usize,
    pub recent_tracks_scroll_offset: usize,
    pub recent_tracks_loading: bool,
    pub(super) recent_tracks_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) recent_tracks_rx:
        Option<std::sync::mpsc::Receiver<Result<Vec<SpotifyTrack>, SpotifyError>>>,

    pub albums: Vec<SpotifyAlbum>,
    pub albums_selected: usize,
    pub albums_scroll_offset: usize,
    pub albums_loading: bool,
    pub albums_has_more: bool,
    pub albums_offset: usize,
    pub open_album: Option<SpotifyAlbum>,
    pub album_tracks: Vec<SpotifyTrack>,
    pub album_tracks_selected: usize,
    pub album_tracks_scroll_offset: usize,
    pub album_tracks_loading: bool,
    pub(super) albums_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) albums_rx: Option<AlbumsResultRx>,
    pub(super) album_tracks_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) album_tracks_rx:
        Option<std::sync::mpsc::Receiver<Result<Vec<SpotifyTrack>, SpotifyError>>>,
}

impl SpotifyState {
    pub fn cleanup(&mut self) {
        fn abort(h: &mut Option<tokio::task::JoinHandle<()>>) {
            if let Some(t) = h.take() {
                t.abort();
            }
        }
        abort(&mut self.auth_task);
        abort(&mut self.search_task);
        abort(&mut self.search_more_task);
        abort(&mut self.devices_task);
        abort(&mut self.playback_task);
        abort(&mut self.token_refresh_task);
        abort(&mut self.radio_task);
        abort(&mut self.liked_task);
        abort(&mut self.playlists_task);
        abort(&mut self.playlist_tracks_task);
        abort(&mut self.top_tracks_task);
        abort(&mut self.recent_tracks_task);
        abort(&mut self.albums_task);
        abort(&mut self.album_tracks_task);
    }
}

impl Default for SpotifyState {
    fn default() -> Self {
        Self {
            status: SpotifyAuthStatus::Idle,
            is_premium: false,
            access_token: None,
            now_playing: None,
            player_status: SpotifyPlayerStatus::Idle,
            active_device_id: None,
            native_available: false,
            native_error: None,
            sub_tab: SpotifySubTab::default(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_loading: false,
            search_loading_more: false,
            search_selected: 0,
            search_scroll_offset: 0,
            search_offset: 0,
            search_has_more: false,
            search_rate_limited: false,
            search_generation: 0,
            rate_limited_until: None,
            volume_pending_until: None,
            devices: Vec::new(),
            devices_loading: false,
            playback: None,
            active_backend: None,
            player_tx: None,
            player_rx: None,
            auth_task: None,
            auth_rx: None,
            search_task: None,
            search_rx: None,
            search_more_task: None,
            search_more_rx: None,
            devices_task: None,
            devices_rx: None,
            playback_task: None,
            playback_rx: None,
            token_refreshed_at: None,
            token_refresh_task: None,
            token_refresh_rx: None,
            play_result_rx: None,
            save_track_rx: None,
            playback_queue: VecDeque::new(),
            radio_queue: VecDeque::new(),
            recently_played: VecDeque::new(),
            radio_task: None,
            radio_rx: None,
            liked_tracks: Vec::new(),
            liked_selected: 0,
            liked_scroll_offset: 0,
            liked_loading: false,
            liked_has_more: false,
            liked_offset: 0,
            liked_rate_limited_until: None,
            liked_task: None,
            liked_rx: None,
            playlists: Vec::new(),
            playlists_selected: 0,
            playlists_scroll_offset: 0,
            playlists_loading: false,
            playlists_has_more: false,
            playlists_offset: 0,
            open_playlist: None,
            playlist_tracks: Vec::new(),
            playlist_tracks_selected: 0,
            playlist_tracks_scroll_offset: 0,
            playlist_tracks_loading: false,
            playlist_tracks_has_more: false,
            playlist_tracks_offset: 0,
            playlists_task: None,
            playlists_rx: None,
            playlist_tracks_task: None,
            playlist_tracks_rx: None,
            top_tracks: Vec::new(),
            top_tracks_selected: 0,
            top_tracks_scroll_offset: 0,
            top_tracks_loading: false,
            top_tracks_task: None,
            top_tracks_rx: None,
            recent_tracks: Vec::new(),
            recent_tracks_selected: 0,
            recent_tracks_scroll_offset: 0,
            recent_tracks_loading: false,
            recent_tracks_task: None,
            recent_tracks_rx: None,
            albums: Vec::new(),
            albums_selected: 0,
            albums_scroll_offset: 0,
            albums_loading: false,
            albums_has_more: false,
            albums_offset: 0,
            open_album: None,
            album_tracks: Vec::new(),
            album_tracks_selected: 0,
            album_tracks_scroll_offset: 0,
            album_tracks_loading: false,
            albums_task: None,
            albums_rx: None,
            album_tracks_task: None,
            album_tracks_rx: None,
        }
    }
}
