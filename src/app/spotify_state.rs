use crate::integrations::spotify::{
    SpotifyError, SpotifyPlaybackState, SpotifyTrack,
    devices::SpotifyDevice,
    player::SpotifyPlayerHandle,
    SpotifyPlayerEvent, AuthResult,
};
use super::modal::{SpotifyAuthStatus, SpotifyPlayerStatus};

pub struct SpotifyState {
    pub status:               SpotifyAuthStatus,
    pub is_premium:           bool,
    pub access_token:         Option<String>,
    pub now_playing:          Option<SpotifyTrack>,
    pub player_status:        SpotifyPlayerStatus,
    pub active_device_id:     Option<String>,

    pub search_query:         String,
    pub search_results:       Vec<SpotifyTrack>,
    pub search_loading:       bool,
    pub search_selected:      usize,
    pub search_offset:        usize,
    pub search_has_more:      bool,
    pub search_rate_limited:  bool,
    pub rate_limited_until:   Option<std::time::Instant>,
    pub volume_pending_until: Option<std::time::Instant>,

    pub devices:              Vec<SpotifyDevice>,
    pub devices_selected:     usize,
    pub devices_loading:      bool,

    pub playback:             Option<SpotifyPlaybackState>,

    pub(super) player_tx:        Option<SpotifyPlayerHandle>,
    pub(super) player_rx:        Option<std::sync::mpsc::Receiver<SpotifyPlayerEvent>>,
    pub(super) auth_task:        Option<tokio::task::JoinHandle<()>>,
    pub(super) auth_rx:          Option<std::sync::mpsc::Receiver<AuthResult>>,
    pub(super) search_task:      Option<tokio::task::JoinHandle<()>>,
    pub(super) search_rx:        Option<std::sync::mpsc::Receiver<(Vec<SpotifyTrack>, bool, Option<u64>)>>,
    pub(super) search_more_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) search_more_rx:   Option<std::sync::mpsc::Receiver<(Vec<SpotifyTrack>, bool, Option<u64>)>>,
    pub(super) devices_task:     Option<tokio::task::JoinHandle<()>>,
    pub(super) devices_rx:       Option<std::sync::mpsc::Receiver<Result<Vec<SpotifyDevice>, SpotifyError>>>,
    pub(super) playback_task:    Option<tokio::task::JoinHandle<()>>,
    pub(super) playback_rx:      Option<std::sync::mpsc::Receiver<Option<SpotifyPlaybackState>>>,
}

impl SpotifyState {
    pub fn cleanup(&mut self) {
        fn abort(h: &mut Option<tokio::task::JoinHandle<()>>) {
            if let Some(t) = h.take() { t.abort(); }
        }
        abort(&mut self.auth_task);
        abort(&mut self.search_task);
        abort(&mut self.search_more_task);
        abort(&mut self.devices_task);
        abort(&mut self.playback_task);
    }
}

impl Default for SpotifyState {
    fn default() -> Self {
        Self {
            status:              SpotifyAuthStatus::Idle,
            is_premium:          false,
            access_token:        None,
            now_playing:         None,
            player_status:       SpotifyPlayerStatus::Idle,
            active_device_id:    None,
            search_query:        String::new(),
            search_results:      Vec::new(),
            search_loading:      false,
            search_selected:     0,
            search_offset:       0,
            search_has_more:     false,
            search_rate_limited:  false,
            rate_limited_until:   None,
            volume_pending_until: None,
            devices:             Vec::new(),
            devices_selected:    0,
            devices_loading:     false,
            playback:            None,
            player_tx:           None,
            player_rx:           None,
            auth_task:           None,
            auth_rx:             None,
            search_task:         None,
            search_rx:           None,
            search_more_task:    None,
            search_more_rx:      None,
            devices_task:        None,
            devices_rx:          None,
            playback_task:       None,
            playback_rx:         None,
        }
    }
}
