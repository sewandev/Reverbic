use crossterm::event::KeyCode;

use super::{abort_task, App};
use super::modal::{IntegrationView, SearchMode, SpotifyAuthStatus, SpotifyField};

impl App {
    pub fn init_integrations(&mut self) {}

    pub(super) fn on_key_modal_integrations(&mut self, key: KeyCode) {
        match self.integration_view {
            IntegrationView::ServiceList       => self.on_key_integration_list(key),
            IntegrationView::SpotifyDetail     => self.on_key_integration_spotify_detail(key),
            IntegrationView::SpotifyUserPass   => self.on_key_integration_spotify_userpass(key),
            IntegrationView::SpotifyWebBrowser => self.on_key_integration_spotify_web(key),
        }
    }

    fn on_key_integration_list(&mut self, key: KeyCode) {
        use super::cycle_next;
        use super::cycle_prev;
        match key {
            KeyCode::Esc => self.modal_mode = SearchMode::Name,
            KeyCode::Up  => self.integration_selected = cycle_prev(self.integration_selected, 1),
            KeyCode::Down => self.integration_selected = cycle_next(self.integration_selected, 1),
            KeyCode::Enter if self.integration_selected == 0 => {
                self.integration_view = IntegrationView::SpotifyDetail;
            }
            _ => {}
        }
    }

    fn on_key_integration_spotify_detail(&mut self, key: KeyCode) {
        use super::cycle_next;
        use super::cycle_prev;

        if matches!(self.spotify_status, SpotifyAuthStatus::LoggedIn) {
            match key {
                KeyCode::Char('d') | KeyCode::Char('D') => self.spotify_logout(),
                KeyCode::Esc => self.integration_view = IntegrationView::ServiceList,
                _ => {}
            }
            return;
        }

        match key {
            KeyCode::Esc => self.integration_view = IntegrationView::ServiceList,
            KeyCode::Char('d') | KeyCode::Char('D')
                if self.config.spotify.display_name.is_some() =>
            {
                self.spotify_logout();
            }
            KeyCode::Up   => self.spotify_auth_selected = cycle_prev(self.spotify_auth_selected, 2),
            KeyCode::Down => self.spotify_auth_selected = cycle_next(self.spotify_auth_selected, 2),
            KeyCode::Enter => match self.spotify_auth_selected {
                0 => self.integration_view = IntegrationView::SpotifyUserPass,
                _ => self.integration_view = IntegrationView::SpotifyWebBrowser,
            },
            _ => {}
        }
    }

    fn on_key_integration_spotify_userpass(&mut self, key: KeyCode) {
        if matches!(self.spotify_status, SpotifyAuthStatus::Connecting) {
            if key == KeyCode::Esc {
                abort_task(&mut self.spotify_auth_task);
                self.spotify_auth_rx = None;
                self.spotify_status  = SpotifyAuthStatus::Idle;
                self.integration_view = IntegrationView::SpotifyDetail;
            }
            return;
        }

        match key {
            KeyCode::Esc => {
                if matches!(self.spotify_status, SpotifyAuthStatus::Error(_)) {
                    self.spotify_status = SpotifyAuthStatus::Idle;
                }
                self.integration_view = IntegrationView::SpotifyDetail;
            }
            KeyCode::Up | KeyCode::Down => {
                self.spotify_field = match self.spotify_field {
                    SpotifyField::Username => SpotifyField::Password,
                    SpotifyField::Password => SpotifyField::Username,
                };
            }
            KeyCode::Enter => {
                if !self.spotify_username_input.is_empty() && !self.spotify_password_input.is_empty() {
                    self.start_spotify_login();
                }
            }
            KeyCode::Backspace => match self.spotify_field {
                SpotifyField::Username => { self.spotify_username_input.pop(); }
                SpotifyField::Password => { self.spotify_password_input.pop(); }
            },
            KeyCode::Char(c) if !c.is_control() => match self.spotify_field {
                SpotifyField::Username => self.spotify_username_input.push(c),
                SpotifyField::Password => self.spotify_password_input.push(c),
            },
            _ => {}
        }
    }

    fn on_key_integration_spotify_web(&mut self, key: KeyCode) {
        if matches!(self.spotify_status, SpotifyAuthStatus::Connecting) {
            if key == KeyCode::Esc {
                abort_task(&mut self.spotify_auth_task);
                self.spotify_auth_rx = None;
                self.spotify_status  = SpotifyAuthStatus::Idle;
            }
            return;
        }
        match key {
            KeyCode::Enter => self.start_oauth_flow(),
            KeyCode::Esc   => self.integration_view = IntegrationView::SpotifyDetail,
            _ => {}
        }
    }

    fn start_oauth_flow(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify_auth_rx = Some(rx);
        self.spotify_status  = SpotifyAuthStatus::Connecting;
        let handle = tokio::spawn(async move {
            let result = crate::integrations::spotify::oauth::start_flow().await;
            let _ = tx.send(result);
        });
        self.spotify_auth_task = Some(handle);
    }

    fn start_spotify_login(&mut self) {
        let username = self.spotify_username_input.clone();
        let password = self.spotify_password_input.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify_auth_rx = Some(rx);
        self.spotify_status  = SpotifyAuthStatus::Connecting;
        let handle = tokio::spawn(async move {
            let result = crate::integrations::spotify::authenticate(username, password).await;
            let _ = tx.send(result);
        });
        self.spotify_auth_task = Some(handle);
    }

    fn spotify_logout(&mut self) {
        self.config.spotify.display_name = None;
        self.config.save();
        self.spotify_status         = SpotifyAuthStatus::Idle;
        self.spotify_username_input.clear();
        self.spotify_password_input.clear();
        self.spotify_field          = SpotifyField::Username;
    }

    pub fn poll_spotify_auth(&mut self) {
        use crate::integrations::spotify::AuthResult;
        if let Some(rx) = self.spotify_auth_rx.take() {
            match rx.try_recv() {
                Ok(AuthResult::Success { username }) => {
                    self.config.spotify.display_name = Some(username);
                    self.config.save();
                    self.spotify_status   = SpotifyAuthStatus::LoggedIn;
                    self.integration_view = IntegrationView::SpotifyDetail;
                    self.spotify_password_input.clear();
                }
                Ok(AuthResult::Failure(msg)) => {
                    self.spotify_status = SpotifyAuthStatus::Error(msg);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify_auth_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify_status = SpotifyAuthStatus::Idle;
                }
            }
        }
    }
}
