use std::collections::VecDeque;

use super::modal::{SpotifyAuthStatus, SpotifyPlayerStatus, SpotifySubTab};
use crate::integrations::spotify::{
    devices::SpotifyDevice, player::SpotifyPlayerHandle, playlists::SpotifyPlaylist, AuthResult,
    SpotifyError, SpotifyPlaybackState, SpotifyPlayerEvent, SpotifyTrack,
};

type SearchPageRx = std::sync::mpsc::Receiver<(Vec<SpotifyTrack>, bool, Option<u64>)>;
type TracksResultRx = std::sync::mpsc::Receiver<Result<(Vec<SpotifyTrack>, bool), SpotifyError>>;
type PlaylistsResultRx =
    std::sync::mpsc::Receiver<Result<(Vec<SpotifyPlaylist>, bool), SpotifyError>>;

pub struct SpotifyState {
    pub status: SpotifyAuthStatus,
    pub is_premium: bool,
    pub access_token: Option<String>,
    pub now_playing: Option<SpotifyTrack>,
    pub player_status: SpotifyPlayerStatus,
    pub active_device_id: Option<String>,

    pub sub_tab: SpotifySubTab,

    pub search_query: String,
    pub search_results: Vec<SpotifyTrack>,
    pub search_loading: bool,
    pub search_loading_more: bool,
    pub search_selected: usize,
    pub search_offset: usize,
    pub search_has_more: bool,
    pub search_rate_limited: bool,
    pub rate_limited_until: Option<std::time::Instant>,
    pub volume_pending_until: Option<std::time::Instant>,

    pub devices: Vec<SpotifyDevice>,
    pub devices_selected: usize,
    pub devices_loading: bool,

    pub playback: Option<SpotifyPlaybackState>,

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

    pub playback_queue: VecDeque<SpotifyTrack>,
    pub radio_queue: VecDeque<SpotifyTrack>,
    pub recently_played: VecDeque<String>,
    pub(super) radio_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) radio_rx: Option<std::sync::mpsc::Receiver<Vec<SpotifyTrack>>>,

    pub liked_tracks: Vec<SpotifyTrack>,
    pub liked_selected: usize,
    pub liked_loading: bool,
    pub liked_has_more: bool,
    pub liked_offset: usize,
    pub(super) liked_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) liked_rx: Option<TracksResultRx>,

    pub playlists: Vec<SpotifyPlaylist>,
    pub playlists_selected: usize,
    pub playlists_loading: bool,
    pub playlists_has_more: bool,
    pub playlists_offset: usize,
    pub open_playlist: Option<SpotifyPlaylist>,
    pub playlist_tracks: Vec<SpotifyTrack>,
    pub playlist_tracks_selected: usize,
    pub playlist_tracks_loading: bool,
    pub playlist_tracks_has_more: bool,
    pub playlist_tracks_offset: usize,
    pub(super) playlists_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playlists_rx: Option<PlaylistsResultRx>,
    pub(super) playlist_tracks_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) playlist_tracks_rx: Option<TracksResultRx>,
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
            sub_tab: SpotifySubTab::default(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_loading: false,
            search_loading_more: false,
            search_selected: 0,
            search_offset: 0,
            search_has_more: false,
            search_rate_limited: false,
            rate_limited_until: None,
            volume_pending_until: None,
            devices: Vec::new(),
            devices_selected: 0,
            devices_loading: false,
            playback: None,
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
            playback_queue: VecDeque::new(),
            radio_queue: VecDeque::new(),
            recently_played: VecDeque::new(),
            radio_task: None,
            radio_rx: None,
            liked_tracks: Vec::new(),
            liked_selected: 0,
            liked_loading: false,
            liked_has_more: false,
            liked_offset: 0,
            liked_task: None,
            liked_rx: None,
            playlists: Vec::new(),
            playlists_selected: 0,
            playlists_loading: false,
            playlists_has_more: false,
            playlists_offset: 0,
            open_playlist: None,
            playlist_tracks: Vec::new(),
            playlist_tracks_selected: 0,
            playlist_tracks_loading: false,
            playlist_tracks_has_more: false,
            playlist_tracks_offset: 0,
            playlists_task: None,
            playlists_rx: None,
            playlist_tracks_task: None,
            playlist_tracks_rx: None,
        }
    }
}
