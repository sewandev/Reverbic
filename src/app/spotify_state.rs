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
pub(super) enum SpotifyRemoteSkipDirection {
    Next,
    Previous,
}

#[derive(Debug)]
pub(super) struct SpotifyRemoteSkipOperation {
    pub(super) id: u64,
    pub(super) device_id: String,
    pub(super) direction: SpotifyRemoteSkipDirection,
}

#[derive(Debug)]
pub(super) struct SpotifyRemoteSkipResult {
    pub(super) id: u64,
    pub(super) device_id: String,
    pub(super) result: Result<(), SpotifyError>,
}

#[derive(Debug, PartialEq, Eq)]
struct SpotifyRemoteSkipInFlight {
    id: u64,
    device_id: String,
}

#[derive(Debug, Default)]
pub(super) struct SpotifyRemoteSkipQueue {
    next_id: u64,
    pending: VecDeque<SpotifyRemoteSkipOperation>,
    in_flight: Option<SpotifyRemoteSkipInFlight>,
}

impl SpotifyRemoteSkipQueue {
    pub(super) fn enqueue(
        &mut self,
        device_id: String,
        direction: SpotifyRemoteSkipDirection,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("Spotify remote skip operation ID exhausted");
        self.pending.push_back(SpotifyRemoteSkipOperation {
            id,
            device_id,
            direction,
        });
        id
    }

    pub(super) fn begin_next(&mut self) -> Option<SpotifyRemoteSkipOperation> {
        if self.in_flight.is_some() {
            return None;
        }
        let operation = self.pending.pop_front()?;
        self.in_flight = Some(SpotifyRemoteSkipInFlight {
            id: operation.id,
            device_id: operation.device_id.clone(),
        });
        Some(operation)
    }

    pub(super) fn complete(&mut self, id: u64, device_id: &str) -> bool {
        let is_current = self
            .in_flight
            .as_ref()
            .is_some_and(|current| current.id == id && current.device_id == device_id);
        if is_current {
            self.in_flight = None;
        }
        is_current
    }

    pub(super) fn abandon_current(&mut self) {
        self.in_flight = None;
    }

    pub(super) fn invalidate(&mut self) {
        self.pending.clear();
        self.in_flight = None;
    }
}

#[cfg(test)]
mod remote_skip_queue_tests {
    use super::*;

    #[test]
    fn remote_skips_are_started_one_at_a_time_in_input_order() {
        let mut queue = SpotifyRemoteSkipQueue::default();
        let first_id = queue.enqueue("device".to_string(), SpotifyRemoteSkipDirection::Next);
        let second_id = queue.enqueue("device".to_string(), SpotifyRemoteSkipDirection::Previous);

        let first = queue.begin_next().expect("first skip starts");
        assert_eq!(first.id, first_id);
        assert_eq!(first.direction, SpotifyRemoteSkipDirection::Next);
        assert!(queue.begin_next().is_none(), "only one skip may run");

        assert!(queue.complete(first.id, &first.device_id));
        let second = queue.begin_next().expect("second skip starts next");
        assert_eq!(second.id, second_id);
        assert_eq!(second.direction, SpotifyRemoteSkipDirection::Previous);
    }

    #[test]
    fn stale_result_cannot_release_the_current_operation() {
        let mut queue = SpotifyRemoteSkipQueue::default();
        queue.enqueue("device".to_string(), SpotifyRemoteSkipDirection::Next);
        let current = queue.begin_next().expect("skip starts");

        assert!(!queue.complete(current.id + 1, &current.device_id));
        assert!(!queue.complete(current.id, "different-device"));
        assert!(queue.begin_next().is_none());
        assert!(queue.complete(current.id, &current.device_id));
    }

    #[test]
    fn invalidating_remote_skips_drops_current_and_pending_operations() {
        let mut queue = SpotifyRemoteSkipQueue::default();
        queue.enqueue("device".to_string(), SpotifyRemoteSkipDirection::Next);
        queue.enqueue("device".to_string(), SpotifyRemoteSkipDirection::Previous);
        queue.begin_next().expect("first skip starts");

        queue.invalidate();

        assert!(queue.begin_next().is_none());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpotifyPlaybackBackend {
    Remote,
    Native,
}

pub struct SpotifyState {
    pub status: SpotifyAuthStatus,
    pub is_premium: Option<bool>,
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
    pub devices_last_fetch: Option<std::time::Instant>,
    pub device_picker_open: bool,
    pub device_picker_selected: usize,
    pub(super) failed_device_ids: std::collections::HashSet<String>,

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
    pub(super) remote_skip_queue: SpotifyRemoteSkipQueue,
    pub(super) remote_skip_paused_for_token_refresh: bool,
    pub(super) remote_skip_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) remote_skip_result_rx: Option<std::sync::mpsc::Receiver<SpotifyRemoteSkipResult>>,
    pub(super) save_track_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,

    pub playback_queue: VecDeque<SpotifyTrack>,
    pub radio_queue: VecDeque<SpotifyTrack>,
    pub(super) native_history: Vec<SpotifyTrack>,
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
        abort(&mut self.remote_skip_task);
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
            is_premium: None,
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
            devices_last_fetch: None,
            device_picker_open: false,
            device_picker_selected: 0,
            failed_device_ids: std::collections::HashSet::new(),
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
            remote_skip_queue: SpotifyRemoteSkipQueue::default(),
            remote_skip_paused_for_token_refresh: false,
            remote_skip_task: None,
            remote_skip_result_rx: None,
            save_track_rx: None,
            playback_queue: VecDeque::new(),
            radio_queue: VecDeque::new(),
            native_history: Vec::new(),
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
