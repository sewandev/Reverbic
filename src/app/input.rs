use std::time::Instant;

use crossterm::event::KeyCode;

use crate::audio::{PlayerCommand, PlayerStatus};
use crate::i18n::t;
use crate::library;
use crate::preview::{deezer_preview, parse_seek_input};
use crate::station::{filter_items, Station, COUNTRIES, GENRES};
use crate::ui::widgets::{
    keep_selected_visible,
    search_modal::{
        modal_tab_at, one_line_list_index_at, radio_favorites_list_area, radio_filter_list_area,
        radio_filtered_results_list_area, radio_playlist_stations_list_area,
        radio_playlists_list_area, radio_search_results_list_area, radio_subtab_at,
        settings_items_area, settings_visible_rows, spotify_auth_notice_at, spotify_body_area,
        spotify_no_device_notice_at, spotify_search_list_area, spotify_subtab_at,
        spotify_titled_track_list_area, two_line_list_index_at, visible_items,
        visible_rows_excluding_scrollbar, youtube_auth_notice_at, youtube_liked_list_area,
        youtube_playlist_videos_list_area, youtube_playlists_list_area, youtube_search_list_area,
        youtube_subtab_at, ListItemHeight,
    },
};

use super::modal::{settings_items, SettingItem};
use super::modal::{
    AppFocus, RadioSubTab, SearchMode, SpotifyAuthStatus, SpotifySubTab, YoutubeSubTab,
};
use super::{abort_task, cycle_next, cycle_prev, scroll_by, App};

fn setting_index_at_visual_row(items: &[SettingItem], visual_row: usize) -> Option<usize> {
    let mut row = 0usize;
    let mut last_group = "";

    for (item_idx, item) in items.iter().enumerate() {
        let group = item.group_key();
        if group != last_group {
            if row == visual_row {
                return None;
            }
            row += 1;
            last_group = group;
        }

        if row == visual_row {
            return Some(item_idx);
        }
        row += 1;
    }

    None
}

fn settings_visual_row_count(items: &[SettingItem]) -> usize {
    let mut rows = 0usize;
    let mut last_group = "";

    for item in items {
        let group = item.group_key();
        if group != last_group {
            rows += 1;
            last_group = group;
        }
        rows += 1;
    }

    rows
}

impl App {
    fn spotify_search_visible_items(&self) -> usize {
        visible_items(
            spotify_search_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        )
    }

    fn youtube_search_visible_items(&self) -> usize {
        visible_items(
            youtube_search_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        )
    }

    fn keep_youtube_search_visible(&mut self) {
        let visible = self.youtube_search_visible_items();
        keep_selected_visible(
            &mut self.youtube.scroll_offset,
            self.youtube.selected,
            visible,
        );
    }

    fn keep_youtube_liked_visible(&mut self) {
        let visible = visible_items(
            youtube_liked_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.youtube.liked_scroll_offset,
            self.youtube.liked_selected,
            visible,
        );
    }

    fn keep_youtube_bookmarks_visible(&mut self) {
        let visible = visible_items(
            youtube_liked_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.youtube.bookmarks_scroll_offset,
            self.youtube.bookmarks_selected,
            visible,
        );
    }

    fn keep_youtube_playlists_visible(&mut self) {
        let visible = visible_items(
            youtube_playlists_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.youtube.playlists_scroll_offset,
            self.youtube.playlists_selected,
            visible,
        );
    }

    fn keep_youtube_playlist_videos_visible(&mut self) {
        let visible = visible_items(
            youtube_playlist_videos_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.youtube.playlist_videos_scroll_offset,
            self.youtube.playlist_videos_selected,
            visible,
        );
    }

    fn radio_search_results_visible_items(&self) -> usize {
        visible_rows_excluding_scrollbar(radio_search_results_list_area(self.terminal_area))
    }

    fn radio_filtered_results_visible_items(&self) -> usize {
        visible_rows_excluding_scrollbar(radio_filtered_results_list_area(self.terminal_area))
    }

    fn keep_radio_favorites_visible(&mut self) {
        let visible = visible_items(
            radio_favorites_list_area(self.terminal_area),
            ListItemHeight::OneLine,
        );
        keep_selected_visible(
            &mut self.radio_fav_scroll_offset,
            self.radio_fav_selected,
            visible,
        );
    }

    fn keep_radio_search_results_visible(&mut self) {
        let visible = self.radio_search_results_visible_items();
        keep_selected_visible(
            &mut self.radio_search_scroll_offset,
            self.modal_selected,
            visible,
        );
    }

    pub(super) fn keep_radio_playlists_visible(&mut self) {
        let visible = visible_items(
            radio_playlists_list_area(self.terminal_area),
            ListItemHeight::OneLine,
        );
        keep_selected_visible(
            &mut self.radio_playlist_scroll_offset,
            self.radio_playlist_selected,
            visible,
        );
    }

    pub(super) fn keep_radio_playlist_stations_visible(&mut self) {
        let visible = visible_items(
            radio_playlist_stations_list_area(self.terminal_area),
            ListItemHeight::OneLine,
        );
        keep_selected_visible(
            &mut self.radio_playlist_station_scroll_offset,
            self.radio_playlist_station_selected,
            visible,
        );
    }

    fn keep_radio_genre_results_visible(&mut self) {
        let visible = self.radio_filtered_results_visible_items();
        keep_selected_visible(
            &mut self.radio_genre_results_scroll_offset,
            self.modal_selected,
            visible,
        );
    }

    fn keep_radio_country_results_visible(&mut self) {
        let visible = self.radio_filtered_results_visible_items();
        keep_selected_visible(
            &mut self.radio_country_results_scroll_offset,
            self.modal_selected,
            visible,
        );
    }

    fn radio_filter_visible_items(&self) -> usize {
        visible_rows_excluding_scrollbar(radio_filter_list_area(self.terminal_area))
    }

    fn keep_genre_filter_visible(&mut self) {
        let visible = self.radio_filter_visible_items();
        keep_selected_visible(
            &mut self.genre_filter_scroll_offset,
            self.genre_selected,
            visible,
        );
    }

    fn keep_country_filter_visible(&mut self) {
        let visible = self.radio_filter_visible_items();
        keep_selected_visible(
            &mut self.country_filter_scroll_offset,
            self.country_selected,
            visible,
        );
    }

    fn settings_visible_items(&self) -> usize {
        settings_visible_rows(self.terminal_area)
    }

    fn settings_selected_row(&self) -> usize {
        let mut row = 0usize;
        let mut last_group = "";
        for (item_idx, item) in
            settings_items(self.config.duck_enabled, self.config.screensaver_secs > 0)
                .iter()
                .enumerate()
        {
            let group = item.group_key();
            if group != last_group {
                row += 1;
                last_group = group;
            }
            if item_idx == self.settings_selected {
                return row;
            }
            row += 1;
        }
        0
    }

    fn keep_settings_visible(&mut self) {
        let selected_row = self.settings_selected_row();
        let visible = self.settings_visible_items();
        keep_selected_visible(&mut self.settings_scroll_offset, selected_row, visible);
    }

    fn keep_active_radio_results_visible(&mut self) {
        match self.modal_mode {
            SearchMode::Genre => self.keep_radio_genre_results_visible(),
            SearchMode::Country => self.keep_radio_country_results_visible(),
            _ => self.keep_radio_search_results_visible(),
        }
    }

    fn reset_active_radio_results_offset(&mut self) {
        match self.modal_mode {
            SearchMode::Genre => self.radio_genre_results_scroll_offset = 0,
            SearchMode::Country => self.radio_country_results_scroll_offset = 0,
            _ => self.radio_search_scroll_offset = 0,
        }
    }

    fn reset_radio_results_offsets(&mut self) {
        self.radio_search_scroll_offset = 0;
        self.radio_genre_results_scroll_offset = 0;
        self.radio_country_results_scroll_offset = 0;
    }

    fn keep_spotify_liked_visible(&mut self) {
        let visible = visible_items(
            spotify_body_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.liked_scroll_offset,
            self.spotify.liked_selected,
            visible,
        );
    }

    fn keep_spotify_playlists_visible(&mut self) {
        let visible = visible_items(
            spotify_body_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.playlists_scroll_offset,
            self.spotify.playlists_selected,
            visible,
        );
    }

    fn keep_spotify_playlist_tracks_visible(&mut self) {
        let visible = visible_items(
            spotify_titled_track_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.playlist_tracks_scroll_offset,
            self.spotify.playlist_tracks_selected,
            visible,
        );
    }

    fn keep_spotify_search_visible(&mut self) {
        let visible = self.spotify_search_visible_items();
        keep_selected_visible(
            &mut self.spotify.search_scroll_offset,
            self.spotify.search_selected,
            visible,
        );
    }

    fn keep_spotify_top_tracks_visible(&mut self) {
        let visible = visible_items(
            spotify_body_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.top_tracks_scroll_offset,
            self.spotify.top_tracks_selected,
            visible,
        );
    }

    fn keep_spotify_recent_tracks_visible(&mut self) {
        let visible = visible_items(
            spotify_body_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.recent_tracks_scroll_offset,
            self.spotify.recent_tracks_selected,
            visible,
        );
    }

    fn keep_spotify_albums_visible(&mut self) {
        let visible = visible_items(
            spotify_body_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.albums_scroll_offset,
            self.spotify.albums_selected,
            visible,
        );
    }

    fn keep_spotify_album_tracks_visible(&mut self) {
        let visible = visible_items(
            spotify_titled_track_list_area(self.terminal_area),
            ListItemHeight::TwoLines,
        );
        keep_selected_visible(
            &mut self.spotify.album_tracks_scroll_offset,
            self.spotify.album_tracks_selected,
            visible,
        );
    }

    pub async fn on_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;

        if event.modifiers.contains(KeyModifiers::CONTROL)
            || event.modifiers.contains(KeyModifiers::SUPER)
        {
            match event.code {
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        if let Ok(text) = clipboard.get_text() {
                            self.on_paste(text);
                        }
                    }
                    return;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if let Some(text) = self.active_copy_text() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(text);
                        }
                        return;
                    }
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.should_quit = true;
                    }
                    return;
                }
                _ => {}
            }
        }

        if event.modifiers.contains(KeyModifiers::SHIFT)
            && !self.show_search_modal
            && matches!(self.focus, AppFocus::Stations)
        {
            if let Some(idx) = self.favorite_index() {
                match event.code {
                    KeyCode::Up => {
                        crate::favorites::move_up(&mut self.favorites, idx);
                        crate::favorites::save(&self.favorites);
                        if idx > 0 {
                            self.selected -= 1;
                        }
                        return;
                    }
                    KeyCode::Down => {
                        crate::favorites::move_down(&mut self.favorites, idx);
                        crate::favorites::save(&self.favorites);
                        if idx + 1 < self.favorites.len() {
                            self.selected += 1;
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }

        if event.modifiers.contains(KeyModifiers::SHIFT)
            && self.show_search_modal
            && matches!(self.modal_mode, SearchMode::Name)
            && matches!(self.radio_sub_tab, RadioSubTab::Favorites)
        {
            let idx = self.radio_fav_selected;
            match event.code {
                KeyCode::Up => {
                    crate::favorites::move_up(&mut self.favorites, idx);
                    crate::favorites::save(&self.favorites);
                    if idx > 0 {
                        self.radio_fav_selected -= 1;
                        self.keep_radio_favorites_visible();
                    }
                    return;
                }
                KeyCode::Down => {
                    crate::favorites::move_down(&mut self.favorites, idx);
                    crate::favorites::save(&self.favorites);
                    if idx + 1 < self.favorites.len() {
                        self.radio_fav_selected += 1;
                        self.keep_radio_favorites_visible();
                    }
                    return;
                }
                _ => {}
            }
        }

        if event.modifiers.contains(KeyModifiers::SHIFT)
            && !event.modifiers.contains(KeyModifiers::CONTROL)
            && self.show_search_modal
            && matches!(self.modal_mode, SearchMode::Name)
            && matches!(self.radio_sub_tab, RadioSubTab::Playlists)
        {
            if let Some(pl_idx) = self.radio_open_playlist {
                let idx = self.radio_playlist_station_selected;
                match event.code {
                    KeyCode::Up => {
                        self.move_playlist_station(pl_idx, idx, -1);
                        return;
                    }
                    KeyCode::Down => {
                        self.move_playlist_station(pl_idx, idx, 1);
                        return;
                    }
                    _ => {}
                }
            }
        }

        if self.spotify.device_picker_open {
            self.on_key_device_picker(event.code).await;
            return;
        }

        if self.playlist_picker.is_some() {
            self.on_key_playlist_picker(event.code);
            return;
        }

        if self.renaming_favorite.is_some() {
            self.on_key_rename(event.code);
            return;
        }

        if self.renaming_playlist.is_some() {
            self.on_key_rename_playlist(event.code);
            return;
        }

        if self.editing_client_id {
            self.on_key_client_id_input(event.code);
            return;
        }

        if self.editing_cookies_path {
            self.on_key_cookies_path_input(event.code);
            return;
        }

        if self.theme_picker_open {
            self.on_key_theme_picker(event.code);
            return;
        }

        if event.modifiers.contains(KeyModifiers::CONTROL)
            && event.modifiers.contains(KeyModifiers::SHIFT)
        {
            match event.code {
                KeyCode::Right => {
                    self.playlist_jump(1).await;
                    return;
                }
                KeyCode::Left => {
                    self.playlist_jump(-1).await;
                    return;
                }
                _ => {}
            }
        }

        if event.modifiers.contains(KeyModifiers::ALT) {
            self.handle_alt_key(event.code).await;
            return;
        }

        if event.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('d') | KeyCode::Char('D') = event.code {
                if self.show_search_modal && matches!(self.modal_mode, SearchMode::Spotify) {
                    self.open_spotify_device_picker();
                    return;
                }
            }
            if let KeyCode::Char('r') | KeyCode::Char('R') = event.code {
                if self.show_search_modal && matches!(self.modal_mode, SearchMode::Youtube) {
                    self.start_youtube_mix();
                    return;
                }
            }
        }

        if !self.show_search_modal {
            let chapter_direction = match event.code {
                KeyCode::Char('[') => Some(-1),
                KeyCode::Char(']') => Some(1),
                _ => None,
            };
            if let Some(direction) = chapter_direction {
                if self.youtube_chapter_jump(direction).await {
                    return;
                }
            }
        }

        self.on_key(event.code).await;
    }

    async fn handle_alt_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.show_search_modal = true;
                self.modal_mode = SearchMode::Settings;
                self.settings_selected = 0;
                self.settings_scroll_offset = 0;
            }
            KeyCode::Char('g') | KeyCode::Char('G') if self.show_search_modal => {
                self.modal_mode = SearchMode::Genre;
                self.genre_filter.clear();
                self.genre_selected = 0;
                self.genre_filter_scroll_offset = 0;
            }
            KeyCode::Char('c') | KeyCode::Char('C') if self.show_search_modal => {
                self.modal_mode = SearchMode::Country;
                self.country_filter.clear();
                self.country_selected = 0;
                self.country_filter_scroll_offset = 0;
            }
            KeyCode::Char('d') | KeyCode::Char('D')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Spotify)
                    && matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) =>
            {
                self.spotify_logout();
                self.modal_mode = SearchMode::Name;
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Spotify)
                    && matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) =>
            {
                self.fetch_spotify_devices();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Name)
                    && matches!(self.radio_sub_tab, RadioSubTab::Favorites) =>
            {
                self.remove_radio_fav_selected();
                self.keep_radio_favorites_visible();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Name)
                    && matches!(self.radio_sub_tab, RadioSubTab::Playlists) =>
            {
                self.remove_playlist_entry_selected();
            }
            KeyCode::Char('p') | KeyCode::Char('P')
                if self.show_search_modal
                    && matches!(
                        self.modal_mode,
                        SearchMode::Name | SearchMode::Genre | SearchMode::Country
                    ) =>
            {
                self.open_playlist_picker_from_context();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.show_search_modal && matches!(self.modal_mode, SearchMode::Youtube) =>
            {
                self.toggle_youtube_bookmark();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.show_search_modal
                    && matches!(
                        self.modal_mode,
                        SearchMode::Name | SearchMode::Genre | SearchMode::Country
                    )
                    && !self.search_results.is_empty() =>
            {
                self.toggle_modal_favorite();
            }
            KeyCode::Char('f') | KeyCode::Char('F') if !self.show_search_modal => {
                self.toggle_selected_favorite();
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if self.show_search_modal
                    && matches!(
                        self.modal_mode,
                        SearchMode::Name | SearchMode::Genre | SearchMode::Country
                    )
                    && !self.search_results.is_empty() =>
            {
                self.last_activity = std::time::Instant::now();
                if let Some(idx) = self.play_random_result() {
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.saved_tracks = Vec::new();
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                if self.show_search_modal && matches!(self.modal_mode, SearchMode::Spotify) {
                    use super::modal::SpotifySubTab;
                    let track = match self.spotify.sub_tab {
                        SpotifySubTab::Search => self
                            .spotify
                            .search_results
                            .get(self.spotify.search_selected)
                            .cloned(),
                        SpotifySubTab::TopTracks => self
                            .spotify
                            .top_tracks
                            .get(self.spotify.top_tracks_selected)
                            .cloned(),
                        SpotifySubTab::Recent => self
                            .spotify
                            .recent_tracks
                            .get(self.spotify.recent_tracks_selected)
                            .cloned(),
                        SpotifySubTab::Liked => self
                            .spotify
                            .liked_tracks
                            .get(self.spotify.liked_selected)
                            .cloned(),
                        SpotifySubTab::Playlists => {
                            if self.spotify.open_playlist.is_some() {
                                self.spotify
                                    .playlist_tracks
                                    .get(self.spotify.playlist_tracks_selected)
                                    .cloned()
                            } else {
                                None
                            }
                        }
                        SpotifySubTab::Albums => {
                            if self.spotify.open_album.is_some() {
                                self.spotify
                                    .album_tracks
                                    .get(self.spotify.album_tracks_selected)
                                    .cloned()
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(t) = track {
                        self.like_spotify_track(&t);
                    }
                } else if let Some(t) = self.spotify.now_playing.clone() {
                    self.like_spotify_track(&t);
                }
            }
            _ => {}
        }
    }

    fn active_copy_text(&self) -> Option<String> {
        if self.renaming_favorite.is_some() || self.renaming_playlist.is_some() {
            return Some(self.rename_input.clone());
        }
        if self.editing_client_id {
            return Some(self.client_id_input.clone());
        }
        if self.editing_cookies_path {
            return Some(self.cookies_path_input.clone());
        }
        if !self.show_search_modal {
            return None;
        }

        match self.modal_mode {
            SearchMode::Name if matches!(self.radio_sub_tab, RadioSubTab::Search) => {
                Some(self.search_query.clone())
            }
            SearchMode::Genre => Some(self.genre_filter.clone()),
            SearchMode::Country => Some(self.country_filter.clone()),
            SearchMode::Spotify if matches!(self.spotify.sub_tab, SpotifySubTab::Search) => {
                Some(self.spotify.search_query.clone())
            }
            SearchMode::Youtube if matches!(self.youtube.sub_tab, YoutubeSubTab::Search) => {
                Some(self.youtube.query.clone())
            }
            _ => None,
        }
    }

    pub fn on_paste(&mut self, text: String) {
        let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
        if filtered.is_empty() {
            return;
        }

        if self.renaming_favorite.is_some() || self.renaming_playlist.is_some() {
            self.rename_input.push_str(&filtered);
            return;
        }
        if self.editing_client_id {
            self.client_id_input.push_str(&filtered);
            return;
        }
        if self.editing_cookies_path {
            self.cookies_path_input.push_str(&filtered);
            self.cookies_path_error = None;
            return;
        }
        if !self.show_search_modal {
            return;
        }

        match self.modal_mode {
            SearchMode::Name if matches!(self.radio_sub_tab, RadioSubTab::Search) => {
                self.search_query.push_str(&filtered);
                self.modal_selected = 0;
                self.radio_search_scroll_offset = 0;
                self.perform_search();
            }
            SearchMode::Genre => {
                self.genre_filter.push_str(&filtered);
                self.genre_selected = 0;
                self.genre_filter_scroll_offset = 0;
            }
            SearchMode::Country => {
                self.country_filter.push_str(&filtered);
                self.country_selected = 0;
                self.country_filter_scroll_offset = 0;
            }
            SearchMode::Spotify if matches!(self.spotify.sub_tab, SpotifySubTab::Search) => {
                self.spotify.search_query.push_str(&filtered);
                self.spotify.search_selected = 0;
                self.spotify.search_scroll_offset = 0;
                self.perform_spotify_search();
            }
            SearchMode::Youtube if matches!(self.youtube.sub_tab, YoutubeSubTab::Search) => {
                self.youtube.query.push_str(&filtered);
                self.youtube.selected = 0;
                self.youtube.scroll_offset = 0;
                self.perform_youtube_search();
            }
            _ => {}
        }
    }

    pub async fn on_key(&mut self, key: KeyCode) {
        if self.screensaver_active() {
            match key {
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    if self.active_source_is_spotify() {
                        self.adjust_spotify_volume(5).await;
                    } else {
                        self.adjust_volume(self.config.volume_step as f32 / 100.0)
                            .await;
                    }
                    return;
                }
                KeyCode::Char('-') => {
                    if self.active_source_is_spotify() {
                        self.adjust_spotify_volume(-5).await;
                    } else {
                        self.adjust_volume(-(self.config.volume_step as f32 / 100.0))
                            .await;
                    }
                    return;
                }
                KeyCode::Char(' ') => {
                    if self.active_source_is_spotify() {
                        self.toggle_spotify_playback().await;
                    } else {
                        match self.player.state().status {
                            PlayerStatus::Playing => {
                                self.player.send(PlayerCommand::Pause).await;
                            }
                            PlayerStatus::Paused => {
                                self.player.send(PlayerCommand::Resume).await;
                            }
                            _ => {}
                        }
                    }
                    return;
                }
                _ => {}
            }
            self.last_activity = Instant::now();
            if key == KeyCode::Char('o') || key == KeyCode::Char('O') {
                if let Some(ref d) = self.station_details {
                    if !d.homepage.is_empty() {
                        crate::shell::open_url(&d.homepage);
                    }
                }
                return;
            }
            if !matches!(key, KeyCode::Enter | KeyCode::Up | KeyCode::Down) {
                return;
            }
        }
        self.last_activity = Instant::now();
        if self.show_search_modal {
            self.on_key_search_modal(key).await;
            return;
        }

        self.clear_notices();
        match key {
            KeyCode::Char(' ') => {
                match self.player.state().status {
                    PlayerStatus::Playing => {
                        self.player.send(PlayerCommand::Pause).await;
                    }
                    PlayerStatus::Paused => {
                        self.player.send(PlayerCommand::Resume).await;
                    }
                    _ => {}
                }
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_volume(self.config.volume_step as f32 / 100.0)
                    .await;
                return;
            }
            KeyCode::Char('-') => {
                self.adjust_volume(-(self.config.volume_step as f32 / 100.0))
                    .await;
                return;
            }
            KeyCode::Char('q') => {
                self.config.last_selected = self.selected;
                self.save_config();
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.should_quit = true;
                return;
            }
            KeyCode::Tab => {
                let has_recent = !self.player.state().recent_titles.is_empty();
                let has_on_demand = !self.on_demand_shows.is_empty() || self.on_demand_loading;

                self.focus = match self.focus {
                    AppFocus::Stations | AppFocus::StationSearch => {
                        if has_on_demand {
                            self.on_demand_selected = 0;
                            AppFocus::OnDemandList
                        } else if has_recent {
                            self.recent_selected = 0;
                            AppFocus::RecentTracks
                        } else {
                            AppFocus::Stations
                        }
                    }
                    AppFocus::OnDemandList => {
                        if has_recent {
                            self.recent_selected = 0;
                            AppFocus::RecentTracks
                        } else {
                            AppFocus::Stations
                        }
                    }
                    AppFocus::RecentTracks => AppFocus::Stations,
                };
                return;
            }
            _ => {}
        }

        match self.focus {
            AppFocus::Stations => self.on_key_stations(key).await,
            AppFocus::RecentTracks => self.on_key_recent(key).await,
            AppFocus::StationSearch => self.on_key_station_search(key).await,
            AppFocus::OnDemandList => self.on_key_on_demand(key).await,
        }
    }

    async fn on_key_search_modal(&mut self, key: KeyCode) {
        if self.show_help {
            self.show_help = false;
            return;
        }
        match key {
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if matches!(self.modal_mode, SearchMode::Spotify) && self.active_source_is_spotify()
                {
                    self.adjust_spotify_volume(5).await;
                } else {
                    self.adjust_volume(self.config.volume_step as f32 / 100.0)
                        .await;
                }
                return;
            }
            KeyCode::Char('-') => {
                if matches!(self.modal_mode, SearchMode::Spotify) && self.active_source_is_spotify()
                {
                    self.adjust_spotify_volume(-5).await;
                } else {
                    self.adjust_volume(-(self.config.volume_step as f32 / 100.0))
                        .await;
                }
                return;
            }
            KeyCode::Char(' ')
                if matches!(self.modal_mode, SearchMode::Name)
                    && matches!(
                        self.radio_sub_tab,
                        RadioSubTab::Favorites | RadioSubTab::Playlists
                    ) =>
            {
                self.toggle_radio_pause().await;
                return;
            }
            KeyCode::Char(' ')
                if matches!(self.modal_mode, SearchMode::Genre | SearchMode::Country)
                    && !self.search_results.is_empty() =>
            {
                self.toggle_radio_pause().await;
                return;
            }
            _ => {}
        }

        if key == KeyCode::Tab {
            let next_mode = match &self.modal_mode {
                SearchMode::Name | SearchMode::Genre | SearchMode::Country => SearchMode::Spotify,
                SearchMode::Spotify => SearchMode::Youtube,
                SearchMode::Youtube => SearchMode::Name,
                other => *other,
            };
            self.switch_modal_mode(next_mode);
            return;
        }

        match self.modal_mode {
            SearchMode::Name => self.on_key_modal_name(key).await,
            SearchMode::Genre => self.on_key_modal_genre(key).await,
            SearchMode::Country => self.on_key_modal_country(key).await,
            SearchMode::Settings => self.on_key_modal_settings(key),
            SearchMode::Spotify => self.on_key_modal_spotify(key).await,
            SearchMode::Youtube => self.on_key_modal_youtube(key).await,
        }
    }

    fn switch_modal_mode(&mut self, mode: SearchMode) {
        self.modal_mode = mode;
        self.modal_selected = 0;
        self.reset_radio_results_offsets();
        self.search_results.clear();
        self.search_query.clear();
        self.genre_filter.clear();
        self.genre_query.clear();
        self.genre_selected = 0;
        self.genre_filter_scroll_offset = 0;
        self.country_filter.clear();
        self.country_selected = 0;
        self.country_filter_scroll_offset = 0;
        abort_task(&mut self.search_task);
        self.search_loading = false;
        self.spotify.search_query.clear();
        self.spotify.search_results.clear();
        self.spotify.search_selected = 0;
        self.spotify.search_scroll_offset = 0;
        abort_task(&mut self.spotify.search_task);
        self.spotify.search_rx = None;
        self.reset_spotify_search_paging();
        self.spotify.search_loading = false;
        self.youtube.query.clear();
        self.youtube.results.clear();
        self.youtube.selected = 0;
        self.youtube.scroll_offset = 0;
        self.youtube.loading = false;
        self.youtube.search_pending_until = None;
        abort_task(&mut self.youtube.search_task);
        self.youtube.search_rx = None;

        if matches!(self.modal_mode, SearchMode::Spotify)
            && matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn)
            && self.spotify.devices.is_empty()
            && !self.spotify.devices_loading
        {
            self.fetch_spotify_devices();
        } else if matches!(self.modal_mode, SearchMode::Youtube) {
            self.ensure_youtube_ready();
        }
    }

    fn modal_tab_is_active(&self, mode: SearchMode) -> bool {
        match mode {
            SearchMode::Name => matches!(
                self.modal_mode,
                SearchMode::Name | SearchMode::Genre | SearchMode::Country
            ),
            SearchMode::Spotify => matches!(self.modal_mode, SearchMode::Spotify),
            SearchMode::Youtube => matches!(self.modal_mode, SearchMode::Youtube),
            SearchMode::Genre | SearchMode::Country | SearchMode::Settings => false,
        }
    }

    async fn on_key_modal_name(&mut self, key: KeyCode) {
        if matches!(key, KeyCode::Left | KeyCode::Right) {
            let next_tab = match (self.radio_sub_tab, key) {
                (RadioSubTab::Search, KeyCode::Right) => RadioSubTab::Favorites,
                (RadioSubTab::Favorites, KeyCode::Right) => RadioSubTab::Playlists,
                (RadioSubTab::Playlists, KeyCode::Right) => RadioSubTab::Search,
                (RadioSubTab::Search, _) => RadioSubTab::Playlists,
                (RadioSubTab::Favorites, _) => RadioSubTab::Search,
                (RadioSubTab::Playlists, _) => RadioSubTab::Favorites,
            };
            self.switch_radio_sub_tab(next_tab);
            return;
        }
        if matches!(self.radio_sub_tab, RadioSubTab::Favorites) {
            self.on_key_radio_favorites(key).await;
            return;
        }
        if matches!(self.radio_sub_tab, RadioSubTab::Playlists) {
            self.on_key_radio_playlists(key).await;
            return;
        }
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                if self.search_query.is_empty() && self.search_results.is_empty() {
                    self.should_quit = true;
                } else {
                    self.search_query.clear();
                    self.search_results.clear();
                    self.modal_selected = 0;
                    self.radio_search_scroll_offset = 0;
                }
            }
            KeyCode::Char('R') if !self.search_results.is_empty() => {
                if let Some(idx) = self.play_random_result() {
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Enter => {
                self.activate_radio_result_selected().await;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.modal_selected = cycle_prev(self.modal_selected, self.search_results.len());
                self.keep_radio_search_results_visible();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.modal_selected = cycle_next(self.modal_selected, self.search_results.len());
                self.keep_radio_search_results_visible();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.modal_selected = 0;
                self.radio_search_scroll_offset = 0;
                self.perform_search();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.search_query.push(c);
                self.modal_selected = 0;
                self.radio_search_scroll_offset = 0;
                self.perform_search();
            }
            _ => {}
        }
    }

    fn switch_radio_sub_tab(&mut self, tab: RadioSubTab) {
        self.radio_sub_tab = tab;
        self.radio_fav_selected = 0;
        self.radio_fav_scroll_offset = 0;
        self.radio_search_scroll_offset = 0;
        self.radio_playlist_selected = 0;
        self.radio_playlist_scroll_offset = 0;
        self.radio_open_playlist = None;
        self.radio_playlist_station_selected = 0;
        self.radio_playlist_station_scroll_offset = 0;
    }

    async fn on_key_radio_favorites(&mut self, key: KeyCode) {
        let len = self.favorites.len();
        match key {
            KeyCode::Esc => {
                self.radio_sub_tab = RadioSubTab::Search;
                self.radio_fav_scroll_offset = 0;
                self.radio_search_scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.radio_fav_selected = super::cycle_prev(self.radio_fav_selected, len);
                self.keep_radio_favorites_visible();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.radio_fav_selected = super::cycle_next(self.radio_fav_selected, len);
                self.keep_radio_favorites_visible();
            }
            KeyCode::Enter => {
                self.activate_radio_favorite_selected().await;
            }
            KeyCode::Char('R') if self.radio_fav_selected < len => {
                self.renaming_favorite = Some(self.radio_fav_selected);
                self.rename_input = self.favorites[self.radio_fav_selected].name.clone();
            }
            _ => {}
        }
    }

    async fn on_key_radio_playlists(&mut self, key: KeyCode) {
        if let Some(pl_idx) = self.radio_open_playlist {
            let len = self
                .playlists
                .get(pl_idx)
                .map(|p| p.stations.len())
                .unwrap_or(0);
            match key {
                KeyCode::Esc => {
                    self.radio_open_playlist = None;
                    self.radio_playlist_station_selected = 0;
                    self.radio_playlist_station_scroll_offset = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.radio_playlist_station_selected =
                        super::cycle_prev(self.radio_playlist_station_selected, len);
                    self.keep_radio_playlist_stations_visible();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.radio_playlist_station_selected =
                        super::cycle_next(self.radio_playlist_station_selected, len);
                    self.keep_radio_playlist_stations_visible();
                }
                KeyCode::Enter => {
                    self.play_playlist_station(pl_idx, self.radio_playlist_station_selected)
                        .await;
                }
                _ => {}
            }
            return;
        }
        let len = self.playlists.len();
        match key {
            KeyCode::Esc => {
                self.radio_sub_tab = RadioSubTab::Search;
                self.radio_playlist_scroll_offset = 0;
                self.radio_search_scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.radio_playlist_selected = super::cycle_prev(self.radio_playlist_selected, len);
                self.keep_radio_playlists_visible();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.radio_playlist_selected = super::cycle_next(self.radio_playlist_selected, len);
                self.keep_radio_playlists_visible();
            }
            KeyCode::Enter if self.radio_playlist_selected < len => {
                self.radio_open_playlist = Some(self.radio_playlist_selected);
                self.radio_playlist_station_selected = 0;
                self.radio_playlist_station_scroll_offset = 0;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.open_new_playlist_input();
            }
            KeyCode::Char('R') if self.radio_playlist_selected < len => {
                self.renaming_playlist = Some(self.radio_playlist_selected);
                self.rename_input = self.playlists[self.radio_playlist_selected].name.clone();
            }
            _ => {}
        }
    }

    async fn on_key_modal_results(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                self.search_results.clear();
                self.genre_query.clear();
                self.modal_selected = 0;
                self.reset_active_radio_results_offset();
            }
            KeyCode::Enter => {
                self.activate_radio_result_selected().await;
            }
            KeyCode::Char('R') => {
                if let Some(idx) = self.play_random_result() {
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.modal_selected = cycle_prev(self.modal_selected, self.search_results.len());
                self.keep_active_radio_results_visible();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.modal_selected = cycle_next(self.modal_selected, self.search_results.len());
                self.keep_active_radio_results_visible();
            }
            _ => {}
        }
    }

    async fn on_key_modal_genre(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            self.on_key_modal_results(key).await;
            return;
        }
        if key == KeyCode::Enter {
            self.activate_genre_filter_selected().await;
            return;
        }
        let filtered = filter_items(GENRES, &self.genre_filter);
        let len = filtered.len();
        let visible = self.radio_filter_visible_items();
        if super::handle_filter_list_key(
            key,
            &mut self.genre_filter,
            &mut self.genre_selected,
            &mut self.genre_filter_scroll_offset,
            len,
            visible,
        ) {
            self.modal_mode = SearchMode::Name;
        }
    }

    async fn on_key_modal_country(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            self.on_key_modal_results(key).await;
            return;
        }
        if key == KeyCode::Enter {
            self.activate_country_filter_selected().await;
            return;
        }
        let filtered = filter_items(COUNTRIES, &self.country_filter);
        let len = filtered.len();
        let visible = self.radio_filter_visible_items();
        if super::handle_filter_list_key(
            key,
            &mut self.country_filter,
            &mut self.country_selected,
            &mut self.country_filter_scroll_offset,
            len,
            visible,
        ) {
            self.modal_mode = SearchMode::Name;
        }
    }

    fn on_key_modal_settings(&mut self, key: KeyCode) {
        let count =
            settings_items(self.config.duck_enabled, self.config.screensaver_secs > 0).len();
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                self.modal_mode = SearchMode::Name;
                self.settings_scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_selected = cycle_prev(self.settings_selected, count);
                self.keep_settings_visible();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_selected = cycle_next(self.settings_selected, count);
                self.keep_settings_visible();
            }
            KeyCode::Enter => {
                self.activate_setting_selected();
            }
            KeyCode::Char(' ') => {
                self.activate_setting_selected();
            }
            _ => {}
        }
    }

    async fn activate_radio_result_selected(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let idx = self.modal_selected.min(self.search_results.len() - 1);
        if matches!(self.modal_mode, SearchMode::Name) && !self.search_query.is_empty() {
            self.config.add_to_history(self.search_query.clone());
            self.save_config();
        }
        self.play_dynamic_station(idx).await;
    }

    async fn activate_radio_favorite_selected(&mut self) {
        self.play_favorite_station(self.radio_fav_selected).await;
    }

    async fn activate_genre_filter_selected(&mut self) {
        let filtered = filter_items(GENRES, &self.genre_filter);
        if let Some(&(tag, label)) = filtered.get(self.genre_selected) {
            self.genre_query = label.to_string();
            self.modal_selected = 0;
            self.radio_genre_results_scroll_offset = 0;
            self.perform_genre_search(tag);
        }
    }

    async fn activate_country_filter_selected(&mut self) {
        let filtered = filter_items(COUNTRIES, &self.country_filter);
        if let Some(&(tag, label)) = filtered.get(self.country_selected) {
            self.genre_query = label.to_string();
            self.modal_selected = 0;
            self.radio_country_results_scroll_offset = 0;
            self.perform_country_search(tag);
        }
    }

    fn activate_setting_selected(&mut self) {
        let items = settings_items(self.config.duck_enabled, self.config.screensaver_secs > 0);
        if let Some(item) = items.get(self.settings_selected).copied() {
            self.activate_setting_item(item);
            if matches!(self.modal_mode, SearchMode::Settings) {
                self.keep_settings_visible();
            }
        }
    }

    fn activate_setting_item(&mut self, item: SettingItem) {
        match item {
            SettingItem::SpotifyClientId => {
                self.client_id_input = self.config.spotify.client_id.clone();
                self.editing_client_id = true;
            }
            SettingItem::YoutubeCookiesPath => {
                self.cookies_path_input = self
                    .config
                    .youtube
                    .cookies_path
                    .as_ref()
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.cookies_path_error = None;
                self.editing_cookies_path = true;
            }
            SettingItem::Theme => self.open_theme_picker(),
            SettingItem::ReplayOnboarding => {
                self.replay_onboarding = true;
                self.show_search_modal = true;
                self.modal_mode = SearchMode::Name;
                self.settings_scroll_offset = 0;
            }
            SettingItem::YoutubeCookiesValidate => {
                self.validate_youtube_cookies();
            }
            SettingItem::OpenLogs => {
                crate::shell::open_folder(&crate::config::reverbic_dir().join("logs"));
            }
            _ => self.apply_settings_toggle(self.settings_selected),
        }
    }

    fn open_theme_picker(&mut self) {
        self.theme_picker_selected = crate::ui::theme::ThemeId::all()
            .iter()
            .position(|theme| *theme == self.config.theme)
            .unwrap_or(0);
        self.theme_picker_open = true;
    }

    fn on_key_theme_picker(&mut self, key: KeyCode) {
        let themes = crate::ui::theme::ThemeId::all();
        match key {
            KeyCode::Esc | KeyCode::Left => {
                self.theme_picker_open = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.theme_picker_selected = cycle_prev(self.theme_picker_selected, themes.len());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.theme_picker_selected = cycle_next(self.theme_picker_selected, themes.len());
            }
            KeyCode::Enter => {
                if let Some(theme) = themes.get(self.theme_picker_selected).copied() {
                    self.config.theme = theme;
                    self.save_config();
                    if let Some(ref tx) = self.windows_tx {
                        let _ = tx.send(self.config.clone());
                    }
                }
                self.theme_picker_open = false;
            }
            _ => {}
        }
    }

    fn on_key_client_id_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.client_id_input.clear();
                self.editing_client_id = false;
            }
            KeyCode::Enter => {
                self.config.spotify.client_id = self.client_id_input.trim().to_string();
                self.save_config();
                self.client_id_input.clear();
                self.editing_client_id = false;
            }
            KeyCode::Backspace => {
                self.client_id_input.pop();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.client_id_input.push(c);
            }
            _ => {}
        }
    }

    fn on_key_cookies_path_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.cookies_path_input.clear();
                self.cookies_path_error = None;
                self.editing_cookies_path = false;
            }
            KeyCode::Enter => {
                let trimmed = self.cookies_path_input.trim();
                if trimmed.is_empty() {
                    self.config.youtube.cookies_path = None;
                    self.youtube.session_health = None;
                    self.youtube.cookies_invalid = false;
                    self.save_config();
                    self.cookies_path_input.clear();
                    self.cookies_path_error = None;
                    self.editing_cookies_path = false;
                    return;
                }

                match crate::integrations::youtube::cookies::validate_cookies_path(
                    std::path::Path::new(trimmed),
                ) {
                    Ok(path) => {
                        self.config.youtube.cookies_path = Some(path);
                        self.youtube.session_health = None;
                        self.youtube.cookies_invalid = false;
                        self.save_config();
                        self.cookies_path_input.clear();
                        self.cookies_path_error = None;
                        self.editing_cookies_path = false;
                        self.start_youtube_session_health_check();
                    }
                    Err(err) => {
                        self.cookies_path_error = Some(err.to_string());
                    }
                }
            }
            KeyCode::Backspace => {
                self.cookies_path_input.pop();
                self.cookies_path_error = None;
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.cookies_path_input.push(c);
                self.cookies_path_error = None;
            }
            _ => {}
        }
    }

    async fn on_key_stations(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.total_stations() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = self.favorite_index() {
                    self.play_favorite_station(idx).await;
                } else if let Some(idx) = self.search_result_index() {
                    self.play_dynamic_station(idx).await;
                } else if let Some(idx) = self.hardcoded_index() {
                    let station = self.stations[idx].clone();
                    self.play_station(station).await;
                }
            }
            KeyCode::Char('e') => {
                if let Some(idx) = self.favorite_index() {
                    self.rename_input = self.favorites[idx].name.clone();
                    self.renaming_favorite = Some(idx);
                }
            }
            KeyCode::Char('r') => {
                let state = self.player.state();
                if let Some(station) = state.station {
                    self.play_station(station).await;
                }
            }
            KeyCode::Char('/') => {
                self.show_search_modal = true;
                self.modal_mode = SearchMode::Name;
                self.search_query.clear();
                self.search_results.clear();
                self.modal_selected = 0;
                self.reset_radio_results_offsets();
            }
            KeyCode::Right => {
                if !self.on_demand_shows.is_empty() || self.on_demand_loading {
                    self.on_demand_selected = 0;
                    self.focus = AppFocus::OnDemandList;
                }
            }
            KeyCode::Char('s') => {
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.saved_tracks = Vec::new();
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.search_results.clear();
                    self.selected = self.selected.min(self.stations.len().saturating_sub(1));
                } else if matches!(
                    self.player.state().status,
                    PlayerStatus::Error(_) | PlayerStatus::Reconnecting(_)
                ) {
                    self.stop_metadata_polling();
                    self.player.send(PlayerCommand::Stop).await;
                } else {
                    self.config.last_selected =
                        self.selected.min(self.stations.len().saturating_sub(1));
                    self.save_config();
                    self.stop_metadata_polling();
                    self.player.send(PlayerCommand::Stop).await;
                    self.should_quit = true;
                }
            }
            _ => {
                if let KeyCode::Char(c) = key {
                    if c.is_alphanumeric() || c == ' ' || c == '-' {
                        self.focus = AppFocus::StationSearch;
                        self.search_query.push(c);
                        self.selected = self.favorites.len() + self.stations.len();
                        self.perform_search();
                    }
                }
            }
        }
    }

    async fn on_key_station_search(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.search_query.clear();
                self.search_results.clear();
                self.selected = self.selected.min(self.stations.len().saturating_sub(1));
                self.focus = AppFocus::Stations;
            }
            KeyCode::Enter => {
                if let Some(idx) = self.search_result_index() {
                    if idx < self.search_results.len() {
                        self.play_dynamic_station(idx).await;
                    }
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if self.search_query.is_empty() {
                    self.search_results.clear();
                    self.selected = self.selected.min(self.stations.len().saturating_sub(1));
                    self.focus = AppFocus::Stations;
                } else {
                    self.selected = self.favorites.len() + self.stations.len();
                    self.perform_search();
                }
            }
            KeyCode::Char(c) if c.is_alphanumeric() || c == ' ' || c == '-' => {
                self.search_query.push(c);
                self.selected = self.favorites.len() + self.stations.len();
                self.perform_search();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let min = self.favorites.len() + self.stations.len();
                if self.selected > min {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.selected + 1 < self.total_stations() => {
                self.selected += 1;
            }
            _ => {}
        }
    }

    pub async fn on_click(&mut self, col: u16, row: u16) {
        let screensaver_was_active = self.screensaver_active();
        self.last_activity = Instant::now();
        if screensaver_was_active {
            if crate::ui::renderer::ambient::url_hit_at(col, row) {
                if let Some(d) = self.station_details.as_ref() {
                    if !d.homepage.is_empty() {
                        crate::shell::open_url(&d.homepage);
                    }
                }
            }
            return;
        }

        if self.show_search_modal {
            self.on_click_search_modal(col, row).await;
            return;
        }

        let h = self.terminal_area.height;
        if h == 0 {
            return;
        }
        let content_start: u16 = if h >= 11 { 4 } else { 2 };
        let footer_rows: u16 = if h >= 11 { 5 } else { 2 };
        let list_max_row = h.saturating_sub(footer_rows);
        if row < content_start || row >= list_max_row {
            return;
        }
        let item_idx = (row - content_start) as usize;

        match &self.focus {
            AppFocus::StationSearch => {
                let abs_idx = self.favorites.len() + self.stations.len() + item_idx;
                if abs_idx < self.total_stations() {
                    self.selected = abs_idx;
                }
            }
            AppFocus::Stations => {
                if item_idx < self.total_stations() {
                    self.selected = item_idx;
                }
            }
            AppFocus::RecentTracks | AppFocus::OnDemandList => {}
        }
    }

    async fn on_click_search_modal(&mut self, col: u16, row: u16) {
        if let Some(mode) = modal_tab_at(self.terminal_area, col, row, self.tab_dots()) {
            if !self.modal_tab_is_active(mode) {
                self.switch_modal_mode(mode);
            }
            return;
        }

        match self.modal_mode {
            SearchMode::Name => {
                if let Some(tab) = radio_subtab_at(
                    self.terminal_area,
                    col,
                    row,
                    self.favorites.len(),
                    self.playlists.len(),
                ) {
                    if self.radio_sub_tab != tab {
                        self.switch_radio_sub_tab(tab);
                    }
                    return;
                }
                self.on_click_radio_name(col, row).await;
            }
            SearchMode::Genre => self.on_click_radio_genre(col, row).await,
            SearchMode::Country => self.on_click_radio_country(col, row).await,
            SearchMode::Settings => self.on_click_settings(col, row),
            SearchMode::Spotify if matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) => {
                if self.spotify_remote_blocked() {
                    if spotify_no_device_notice_at(self.terminal_area, col, row) {
                        crate::shell::open_url(&t("modal.spotify.auth_notice.guide_url"));
                    }
                    return;
                }
                if let Some(tab) = spotify_subtab_at(self.terminal_area, col, row) {
                    if self.spotify.sub_tab != tab {
                        self.switch_spotify_sub_tab(tab);
                    }
                    return;
                }
                self.on_click_spotify(col, row).await;
            }
            SearchMode::Spotify if matches!(self.spotify.status, SpotifyAuthStatus::Idle) => {
                if spotify_auth_notice_at(self.terminal_area, col, row) {
                    crate::shell::open_url(&t("modal.spotify.auth_notice.guide_url"));
                }
            }
            SearchMode::Youtube => {
                if let Some(tab) = youtube_subtab_at(self.terminal_area, col, row) {
                    if self.youtube.sub_tab != tab {
                        self.switch_youtube_sub_tab(tab);
                    }
                    return;
                }
                if matches!(
                    self.youtube.sub_tab,
                    YoutubeSubTab::Liked | YoutubeSubTab::Playlists
                ) && (self.config.youtube.cookies_path.is_none() || self.youtube.cookies_invalid)
                    && youtube_auth_notice_at(self.terminal_area, col, row)
                {
                    crate::shell::open_url(&t("modal.youtube.auth_notice.guide_url"));
                    return;
                }
                self.on_click_youtube(col, row).await;
            }
            _ => {}
        }
    }

    async fn on_click_radio_name(&mut self, col: u16, row: u16) {
        match self.radio_sub_tab {
            RadioSubTab::Search => self.on_click_radio_search_results(col, row).await,
            RadioSubTab::Favorites => self.on_click_radio_favorites(col, row).await,
            RadioSubTab::Playlists => self.on_click_radio_playlists(col, row).await,
        }
    }

    async fn on_click_radio_search_results(&mut self, col: u16, row: u16) {
        let len = self.search_results.len();
        let Some(idx) = one_line_list_index_at(
            radio_search_results_list_area(self.terminal_area),
            col,
            row,
            self.modal_selected,
            self.radio_search_results_visible_items(),
            self.radio_search_scroll_offset,
            len,
        ) else {
            return;
        };

        self.modal_selected = idx;
        self.activate_radio_result_selected().await;
    }

    async fn on_click_radio_favorites(&mut self, col: u16, row: u16) {
        let len = self.favorites.len();
        let visible = visible_items(
            radio_favorites_list_area(self.terminal_area),
            ListItemHeight::OneLine,
        );
        let Some(idx) = one_line_list_index_at(
            radio_favorites_list_area(self.terminal_area),
            col,
            row,
            self.radio_fav_selected,
            visible,
            self.radio_fav_scroll_offset,
            len,
        ) else {
            return;
        };

        self.radio_fav_selected = idx;
        self.activate_radio_favorite_selected().await;
    }

    async fn on_click_radio_playlists(&mut self, col: u16, row: u16) {
        if let Some(pl_idx) = self.radio_open_playlist {
            let len = self
                .playlists
                .get(pl_idx)
                .map(|p| p.stations.len())
                .unwrap_or(0);
            let area = radio_playlist_stations_list_area(self.terminal_area);
            let visible = visible_items(area, ListItemHeight::OneLine);
            let Some(idx) = one_line_list_index_at(
                area,
                col,
                row,
                self.radio_playlist_station_selected,
                visible,
                self.radio_playlist_station_scroll_offset,
                len,
            ) else {
                return;
            };
            self.radio_playlist_station_selected = idx;
            self.play_playlist_station(pl_idx, idx).await;
            return;
        }

        let len = self.playlists.len();
        let area = radio_playlists_list_area(self.terminal_area);
        let visible = visible_items(area, ListItemHeight::OneLine);
        let Some(idx) = one_line_list_index_at(
            area,
            col,
            row,
            self.radio_playlist_selected,
            visible,
            self.radio_playlist_scroll_offset,
            len,
        ) else {
            return;
        };
        self.radio_playlist_selected = idx;
        self.radio_open_playlist = Some(idx);
        self.radio_playlist_station_selected = 0;
        self.radio_playlist_station_scroll_offset = 0;
    }

    async fn on_click_radio_genre(&mut self, col: u16, row: u16) {
        if !self.search_results.is_empty() {
            self.on_click_radio_filtered_results(col, row, self.radio_genre_results_scroll_offset)
                .await;
            return;
        }

        let filtered = filter_items(GENRES, &self.genre_filter);
        let Some(idx) = one_line_list_index_at(
            radio_filter_list_area(self.terminal_area),
            col,
            row,
            self.genre_selected,
            self.radio_filter_visible_items(),
            self.genre_filter_scroll_offset,
            filtered.len(),
        ) else {
            return;
        };

        self.genre_selected = idx;
        self.activate_genre_filter_selected().await;
    }

    async fn on_click_radio_country(&mut self, col: u16, row: u16) {
        if !self.search_results.is_empty() {
            self.on_click_radio_filtered_results(
                col,
                row,
                self.radio_country_results_scroll_offset,
            )
            .await;
            return;
        }

        let filtered = filter_items(COUNTRIES, &self.country_filter);
        let Some(idx) = one_line_list_index_at(
            radio_filter_list_area(self.terminal_area),
            col,
            row,
            self.country_selected,
            self.radio_filter_visible_items(),
            self.country_filter_scroll_offset,
            filtered.len(),
        ) else {
            return;
        };

        self.country_selected = idx;
        self.activate_country_filter_selected().await;
    }

    async fn on_click_radio_filtered_results(&mut self, col: u16, row: u16, scroll_offset: usize) {
        let len = self.search_results.len();
        let Some(idx) = one_line_list_index_at(
            radio_filtered_results_list_area(self.terminal_area),
            col,
            row,
            self.modal_selected,
            self.radio_filtered_results_visible_items(),
            scroll_offset,
            len,
        ) else {
            return;
        };

        self.modal_selected = idx;
        self.activate_radio_result_selected().await;
    }

    fn on_click_settings(&mut self, col: u16, row: u16) {
        let items = settings_items(self.config.duck_enabled, self.config.screensaver_secs > 0);
        let visual_row_count = settings_visual_row_count(&items);
        let Some(visual_row) = one_line_list_index_at(
            settings_items_area(self.terminal_area),
            col,
            row,
            self.settings_selected_row(),
            self.settings_visible_items(),
            self.settings_scroll_offset,
            visual_row_count,
        ) else {
            return;
        };

        let Some(idx) = setting_index_at_visual_row(&items, visual_row) else {
            return;
        };

        self.settings_selected = idx;
        self.activate_setting_selected();
    }

    async fn on_click_spotify(&mut self, col: u16, row: u16) {
        match self.spotify.sub_tab {
            SpotifySubTab::Search => self.on_click_spotify_search(col, row).await,
            SpotifySubTab::Liked => self.on_click_spotify_liked(col, row).await,
            SpotifySubTab::Playlists => self.on_click_spotify_playlists(col, row).await,
            SpotifySubTab::TopTracks => self.on_click_spotify_top_tracks(col, row).await,
            SpotifySubTab::Recent => self.on_click_spotify_recent_tracks(col, row).await,
            SpotifySubTab::Albums => self.on_click_spotify_albums(col, row).await,
        }
    }

    async fn on_click_spotify_search(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_search_list_area(self.terminal_area),
            col,
            row,
            self.spotify.search_selected,
            self.spotify_search_visible_items(),
            self.spotify.search_scroll_offset,
            self.spotify.search_results.len(),
        ) else {
            return;
        };

        self.spotify.search_selected = idx;
        self.activate_spotify_search_selected().await;
    }

    async fn on_click_spotify_liked(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_body_area(self.terminal_area),
            col,
            row,
            self.spotify.liked_selected,
            visible_items(
                spotify_body_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.liked_scroll_offset,
            self.spotify.liked_tracks.len(),
        ) else {
            return;
        };

        self.spotify.liked_selected = idx;
        self.activate_spotify_liked_selected().await;
    }

    async fn on_click_spotify_playlists(&mut self, col: u16, row: u16) {
        if self.spotify.open_playlist.is_some() {
            self.on_click_spotify_playlist_tracks(col, row).await;
        } else {
            self.on_click_spotify_playlist_list(col, row).await;
        }
    }

    async fn on_click_spotify_playlist_list(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_body_area(self.terminal_area),
            col,
            row,
            self.spotify.playlists_selected,
            visible_items(
                spotify_body_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.playlists_scroll_offset,
            self.spotify.playlists.len(),
        ) else {
            return;
        };

        self.spotify.playlists_selected = idx;
        self.activate_spotify_playlist_selected().await;
    }

    async fn on_click_spotify_playlist_tracks(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_titled_track_list_area(self.terminal_area),
            col,
            row,
            self.spotify.playlist_tracks_selected,
            visible_items(
                spotify_titled_track_list_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.playlist_tracks_scroll_offset,
            self.spotify.playlist_tracks.len(),
        ) else {
            return;
        };

        self.spotify.playlist_tracks_selected = idx;
        self.activate_spotify_playlist_track_selected().await;
    }

    async fn on_click_spotify_top_tracks(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_body_area(self.terminal_area),
            col,
            row,
            self.spotify.top_tracks_selected,
            visible_items(
                spotify_body_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.top_tracks_scroll_offset,
            self.spotify.top_tracks.len(),
        ) else {
            return;
        };

        self.spotify.top_tracks_selected = idx;
        self.activate_spotify_top_track_selected().await;
    }

    async fn on_click_spotify_recent_tracks(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_body_area(self.terminal_area),
            col,
            row,
            self.spotify.recent_tracks_selected,
            visible_items(
                spotify_body_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.recent_tracks_scroll_offset,
            self.spotify.recent_tracks.len(),
        ) else {
            return;
        };

        self.spotify.recent_tracks_selected = idx;
        self.activate_spotify_recent_track_selected().await;
    }

    async fn on_click_spotify_albums(&mut self, col: u16, row: u16) {
        if self.spotify.open_album.is_some() {
            self.on_click_spotify_album_tracks(col, row).await;
        } else {
            self.on_click_spotify_album_list(col, row).await;
        }
    }

    async fn on_click_spotify_album_list(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_body_area(self.terminal_area),
            col,
            row,
            self.spotify.albums_selected,
            visible_items(
                spotify_body_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.albums_scroll_offset,
            self.spotify.albums.len(),
        ) else {
            return;
        };

        self.spotify.albums_selected = idx;
        self.activate_spotify_album_selected().await;
    }

    async fn on_click_spotify_album_tracks(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            spotify_titled_track_list_area(self.terminal_area),
            col,
            row,
            self.spotify.album_tracks_selected,
            visible_items(
                spotify_titled_track_list_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.spotify.album_tracks_scroll_offset,
            self.spotify.album_tracks.len(),
        ) else {
            return;
        };

        self.spotify.album_tracks_selected = idx;
        self.activate_spotify_album_track_selected().await;
    }

    async fn on_click_youtube(&mut self, col: u16, row: u16) {
        match self.youtube.sub_tab {
            YoutubeSubTab::Search => self.on_click_youtube_search(col, row).await,
            YoutubeSubTab::Bookmarks => self.on_click_youtube_bookmarks(col, row).await,
            YoutubeSubTab::Liked => self.on_click_youtube_liked(col, row).await,
            YoutubeSubTab::Playlists => self.on_click_youtube_playlists(col, row).await,
        }
    }

    async fn on_click_youtube_bookmarks(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            youtube_liked_list_area(self.terminal_area),
            col,
            row,
            self.youtube.bookmarks_selected,
            visible_items(
                youtube_liked_list_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.youtube.bookmarks_scroll_offset,
            self.youtube.bookmarks.len(),
        ) else {
            return;
        };

        self.youtube.bookmarks_selected = idx;
        self.activate_youtube_bookmark_selected().await;
    }

    async fn on_click_youtube_search(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            youtube_search_list_area(self.terminal_area),
            col,
            row,
            self.youtube.selected,
            self.youtube_search_visible_items(),
            self.youtube.scroll_offset,
            self.youtube.results.len(),
        ) else {
            return;
        };

        self.youtube.selected = idx;
        self.activate_youtube_selected().await;
    }

    async fn on_click_youtube_liked(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            youtube_liked_list_area(self.terminal_area),
            col,
            row,
            self.youtube.liked_selected,
            visible_items(
                youtube_liked_list_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.youtube.liked_scroll_offset,
            self.youtube.liked_videos.len(),
        ) else {
            return;
        };

        self.youtube.liked_selected = idx;
        self.activate_youtube_liked_selected().await;
    }

    async fn on_click_youtube_playlists(&mut self, col: u16, row: u16) {
        if self.youtube.open_playlist.is_some() {
            self.on_click_youtube_playlist_videos(col, row).await;
        } else {
            self.on_click_youtube_playlist_list(col, row).await;
        }
    }

    async fn on_click_youtube_playlist_list(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            youtube_playlists_list_area(self.terminal_area),
            col,
            row,
            self.youtube.playlists_selected,
            visible_items(
                youtube_playlists_list_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.youtube.playlists_scroll_offset,
            self.youtube.playlists.len(),
        ) else {
            return;
        };

        self.youtube.playlists_selected = idx;
        self.activate_youtube_playlist_selected().await;
    }

    async fn on_click_youtube_playlist_videos(&mut self, col: u16, row: u16) {
        let Some(idx) = two_line_list_index_at(
            youtube_playlist_videos_list_area(self.terminal_area),
            col,
            row,
            self.youtube.playlist_videos_selected,
            visible_items(
                youtube_playlist_videos_list_area(self.terminal_area),
                ListItemHeight::TwoLines,
            ),
            self.youtube.playlist_videos_scroll_offset,
            self.youtube.playlist_videos.len(),
        ) else {
            return;
        };

        self.youtube.playlist_videos_selected = idx;
        self.activate_youtube_playlist_video_selected().await;
    }

    pub async fn on_mouse_scroll(&mut self, delta: i32) {
        self.last_activity = Instant::now();
        if self.show_search_modal {
            match self.modal_mode {
                SearchMode::Youtube => match self.youtube.sub_tab {
                    YoutubeSubTab::Search => {
                        let len = self.youtube.results.len();
                        if len > 0 {
                            self.youtube.selected = scroll_by(self.youtube.selected, delta, len);
                            self.keep_youtube_search_visible();
                        }
                    }
                    YoutubeSubTab::Bookmarks => {
                        let len = self.youtube.bookmarks.len();
                        if len > 0 {
                            self.youtube.bookmarks_selected =
                                scroll_by(self.youtube.bookmarks_selected, delta, len);
                            self.keep_youtube_bookmarks_visible();
                        }
                    }
                    YoutubeSubTab::Liked => {
                        let len = self.youtube.liked_videos.len();
                        if len > 0 {
                            self.youtube.liked_selected =
                                scroll_by(self.youtube.liked_selected, delta, len);
                            self.keep_youtube_liked_visible();
                        }
                    }
                    YoutubeSubTab::Playlists => {
                        if self.youtube.open_playlist.is_some() {
                            let len = self.youtube.playlist_videos.len();
                            if len > 0 {
                                self.youtube.playlist_videos_selected =
                                    scroll_by(self.youtube.playlist_videos_selected, delta, len);
                                self.keep_youtube_playlist_videos_visible();
                            }
                        } else {
                            let len = self.youtube.playlists.len();
                            if len > 0 {
                                self.youtube.playlists_selected =
                                    scroll_by(self.youtube.playlists_selected, delta, len);
                                self.keep_youtube_playlists_visible();
                            }
                        }
                    }
                },
                SearchMode::Spotify => {
                    use crate::app::SpotifySubTab;
                    match self.spotify.sub_tab {
                        SpotifySubTab::Search => {
                            let len = self.spotify.search_results.len();
                            if len > 0 {
                                self.spotify.search_selected =
                                    scroll_by(self.spotify.search_selected, delta, len);
                                self.keep_spotify_search_visible();
                            }
                        }
                        SpotifySubTab::Liked => {
                            let len = self.spotify.liked_tracks.len();
                            if len > 0 {
                                self.spotify.liked_selected =
                                    scroll_by(self.spotify.liked_selected, delta, len);
                                self.keep_spotify_liked_visible();
                                if delta > 0
                                    && self.spotify.liked_selected >= len - 1
                                    && self.spotify.liked_has_more
                                {
                                    self.load_more_spotify_liked();
                                }
                            }
                        }
                        SpotifySubTab::Playlists => {
                            if self.spotify.open_playlist.is_some() {
                                let len = self.spotify.playlist_tracks.len();
                                if len > 0 {
                                    self.spotify.playlist_tracks_selected = scroll_by(
                                        self.spotify.playlist_tracks_selected,
                                        delta,
                                        len,
                                    );
                                    self.keep_spotify_playlist_tracks_visible();
                                }
                            } else {
                                let len = self.spotify.playlists.len();
                                if len > 0 {
                                    self.spotify.playlists_selected =
                                        scroll_by(self.spotify.playlists_selected, delta, len);
                                    self.keep_spotify_playlists_visible();
                                }
                            }
                        }
                        SpotifySubTab::TopTracks => {
                            let len = self.spotify.top_tracks.len();
                            if len > 0 {
                                self.spotify.top_tracks_selected =
                                    scroll_by(self.spotify.top_tracks_selected, delta, len);
                                self.keep_spotify_top_tracks_visible();
                            }
                        }
                        SpotifySubTab::Recent => {
                            let len = self.spotify.recent_tracks.len();
                            if len > 0 {
                                self.spotify.recent_tracks_selected =
                                    scroll_by(self.spotify.recent_tracks_selected, delta, len);
                                self.keep_spotify_recent_tracks_visible();
                            }
                        }
                        SpotifySubTab::Albums => {
                            if self.spotify.open_album.is_some() {
                                let len = self.spotify.album_tracks.len();
                                if len > 0 {
                                    self.spotify.album_tracks_selected =
                                        scroll_by(self.spotify.album_tracks_selected, delta, len);
                                    self.keep_spotify_album_tracks_visible();
                                }
                            } else {
                                let len = self.spotify.albums.len();
                                if len > 0 {
                                    self.spotify.albums_selected =
                                        scroll_by(self.spotify.albums_selected, delta, len);
                                    self.keep_spotify_albums_visible();
                                }
                            }
                        }
                    }
                }
                _ => {
                    if matches!(self.modal_mode, SearchMode::Name)
                        && matches!(self.radio_sub_tab, RadioSubTab::Favorites)
                    {
                        let len = self.favorites.len();
                        if len > 0 {
                            self.radio_fav_selected =
                                scroll_by(self.radio_fav_selected, delta, len);
                            self.keep_radio_favorites_visible();
                        }
                        return;
                    }

                    if matches!(self.modal_mode, SearchMode::Name)
                        && matches!(self.radio_sub_tab, RadioSubTab::Playlists)
                    {
                        if let Some(pl_idx) = self.radio_open_playlist {
                            let len = self
                                .playlists
                                .get(pl_idx)
                                .map(|p| p.stations.len())
                                .unwrap_or(0);
                            if len > 0 {
                                self.radio_playlist_station_selected =
                                    scroll_by(self.radio_playlist_station_selected, delta, len);
                                self.keep_radio_playlist_stations_visible();
                            }
                        } else {
                            let len = self.playlists.len();
                            if len > 0 {
                                self.radio_playlist_selected =
                                    scroll_by(self.radio_playlist_selected, delta, len);
                                self.keep_radio_playlists_visible();
                            }
                        }
                        return;
                    }

                    if !self.search_results.is_empty()
                        && matches!(
                            self.modal_mode,
                            SearchMode::Name | SearchMode::Genre | SearchMode::Country
                        )
                    {
                        let len = self.search_results.len();
                        self.modal_selected = scroll_by(self.modal_selected, delta, len);
                        self.keep_active_radio_results_visible();
                        return;
                    }

                    match self.modal_mode {
                        SearchMode::Genre if self.search_results.is_empty() => {
                            let len = filter_items(GENRES, &self.genre_filter).len();
                            if len > 0 {
                                self.genre_selected = scroll_by(self.genre_selected, delta, len);
                                self.keep_genre_filter_visible();
                            }
                        }
                        SearchMode::Country if self.search_results.is_empty() => {
                            let len = filter_items(COUNTRIES, &self.country_filter).len();
                            if len > 0 {
                                self.country_selected =
                                    scroll_by(self.country_selected, delta, len);
                                self.keep_country_filter_visible();
                            }
                        }
                        SearchMode::Settings => {
                            let len = settings_items(
                                self.config.duck_enabled,
                                self.config.screensaver_secs > 0,
                            )
                            .len();
                            if len > 0 {
                                self.settings_selected =
                                    scroll_by(self.settings_selected, delta, len);
                                self.keep_settings_visible();
                            }
                        }
                        _ => {
                            let len = self.search_results.len();
                            if len > 0 {
                                self.modal_selected = scroll_by(self.modal_selected, delta, len);
                            }
                        }
                    }
                }
            }
            return;
        }

        match self.focus {
            AppFocus::RecentTracks => {
                let len = self.player.state().recent_titles.len();
                if len == 0 {
                    return;
                }
                self.recent_selected = scroll_by(self.recent_selected, delta, len);
            }
            AppFocus::OnDemandList => {
                let len = self.on_demand_shows.len();
                if len == 0 {
                    return;
                }
                self.on_demand_selected = scroll_by(self.on_demand_selected, delta, len);
            }
            AppFocus::Stations | AppFocus::StationSearch => {
                self.selected = scroll_by(self.selected, delta, self.total_stations());
            }
        }
    }

    pub async fn on_double_click(&mut self) {
        self.last_activity = Instant::now();
        if self.show_search_modal {
            return;
        }
        match self.focus {
            AppFocus::Stations | AppFocus::StationSearch => {
                self.click_flash = Some((self.selected, Instant::now()));
                if let Some(idx) = self.favorite_index() {
                    self.play_favorite_station(idx).await;
                } else if let Some(idx) = self.search_result_index() {
                    self.play_dynamic_station(idx).await;
                } else if let Some(idx) = self.hardcoded_index() {
                    let station = self.stations[idx].clone();
                    self.play_station(station).await;
                }
            }
            _ => {}
        }
    }

    async fn on_key_on_demand(&mut self, key: KeyCode) {
        let len = self.on_demand_shows.len();
        if len == 0 && !self.on_demand_loading {
            self.focus = AppFocus::Stations;
            return;
        }

        match key {
            KeyCode::Char('p') => {
                let total_programs = crate::station::on_demand::PROGRAMS.len();
                self.selected_program = (self.selected_program + 1) % total_programs;
                self.on_demand_selected = 0;
                self.start_on_demand_fetch();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.seek_input.clear();
                if self.on_demand_selected > 0 {
                    self.on_demand_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.seek_input.clear();
                if len > 0 && self.on_demand_selected + 1 < len {
                    self.on_demand_selected += 1;
                }
            }
            KeyCode::Enter => {
                if !self.seek_input.is_empty() {
                    if let Some(target) = parse_seek_input(&self.seek_input) {
                        self.player.send(PlayerCommand::Seek(target)).await;
                    }
                    self.seek_input.clear();
                } else if let Some(show) = self.on_demand_shows.get(self.on_demand_selected) {
                    let station = Station {
                        key: format!("ondemand_{}", show.id),
                        name: show.title.clone(),
                        url: show.audio_url.clone(),
                        metadata_api_url: None,
                        history_api_url: None,
                        schedule_url: None,
                        show_countdown: false,
                        bitrate_kbps: None,
                        custom_headers: None,
                    };
                    self.play_station(station).await;
                }
            }
            KeyCode::Backspace => {
                self.seek_input.pop();
            }
            KeyCode::Char('[') => {
                let pos = self.player.state().playback_pos_secs.unwrap_or(0.0);
                self.player
                    .send(PlayerCommand::Seek((pos - 60.0).max(0.0)))
                    .await;
            }
            KeyCode::Char(']') => {
                let state = self.player.state();
                let pos = state.playback_pos_secs.unwrap_or(0.0);
                let target = pos + 60.0;
                let target = state
                    .playback_duration_secs
                    .map(|d| target.min(d))
                    .unwrap_or(target);
                self.player.send(PlayerCommand::Seek(target)).await;
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == ':' => {
                if self.seek_input.len() < 7 {
                    self.seek_input.push(c);
                }
            }
            KeyCode::Left | KeyCode::Esc => {
                if !self.seek_input.is_empty() {
                    self.seek_input.clear();
                } else {
                    self.focus = AppFocus::Stations;
                }
            }
            _ => {}
        }
    }

    fn apply_settings_toggle(&mut self, idx: usize) {
        use crate::i18n;
        let Some(&item) =
            settings_items(self.config.duck_enabled, self.config.screensaver_secs > 0).get(idx)
        else {
            return;
        };
        match item {
            super::modal::SettingItem::Autoplay => {
                self.config.autoplay_last = !self.config.autoplay_last
            }
            super::modal::SettingItem::RestoreVolume => {
                self.config.restore_volume = !self.config.restore_volume
            }
            super::modal::SettingItem::Crossfade => self.config.crossfade_next(),
            super::modal::SettingItem::YoutubeCrossfade => self.config.youtube_crossfade_next(),
            super::modal::SettingItem::VolumeStep => self.config.volume_step_next(),
            super::modal::SettingItem::Prebuffer => {
                self.config.prebuffer_next();
                let secs = self.config.prebuffer_secs as f32;
                let cmd_tx = self.player.clone_sender();
                tokio::spawn(async move {
                    let _ = cmd_tx
                        .send(crate::audio::PlayerCommand::SetPrebuffer(secs))
                        .await;
                });
            }
            super::modal::SettingItem::OverlayMode => {
                self.config.overlay_mode = self.config.overlay_mode.next()
            }
            super::modal::SettingItem::OverlayAlpha => {
                self.config.overlay_alpha = match self.config.overlay_alpha {
                    v if v < 30 => 30,
                    v if v < 50 => 50,
                    v if v < 70 => 70,
                    v if v < 90 => 90,
                    _ => 20,
                };
            }
            super::modal::SettingItem::OverlayPosition => {
                self.config.overlay_position = self.config.overlay_position.next()
            }
            super::modal::SettingItem::OverlayStyle => {
                self.config.overlay_style = self.config.overlay_style.next()
            }
            super::modal::SettingItem::Screensaver => self.config.screensaver_next(),
            super::modal::SettingItem::ScreensaverClock => {
                self.config.screensaver_clock = !self.config.screensaver_clock
            }
            super::modal::SettingItem::ScreensaverLogo => {
                self.config.screensaver_logo = !self.config.screensaver_logo
            }
            super::modal::SettingItem::ScreensaverVisualizer => {
                self.config.screensaver_visualizer = !self.config.screensaver_visualizer
            }
            super::modal::SettingItem::ScreensaverRecentTracks => {
                self.config.screensaver_recent_tracks = !self.config.screensaver_recent_tracks
            }
            super::modal::SettingItem::ScreensaverProgressBar => {
                self.config.screensaver_progress_bar = !self.config.screensaver_progress_bar
            }
            super::modal::SettingItem::ScreensaverStationDetails => {
                self.config.screensaver_station_details = !self.config.screensaver_station_details
            }
            super::modal::SettingItem::ScreensaverNowPlaying => {
                self.config.screensaver_now_playing = !self.config.screensaver_now_playing
            }
            super::modal::SettingItem::DuckEnabled => {
                self.config.duck_enabled = !self.config.duck_enabled
            }
            super::modal::SettingItem::DuckVolume => {
                self.config.duck_volume = match self.config.duck_volume {
                    v if v < 20 => 20,
                    v if v < 30 => 30,
                    v if v < 40 => 40,
                    v if v < 50 => 50,
                    v if v < 60 => 60,
                    v if v < 70 => 70,
                    v if v < 80 => 80,
                    _ => 10,
                };
            }
            super::modal::SettingItem::MediaKeys => {
                self.config.media_keys = !self.config.media_keys
            }
            super::modal::SettingItem::TrayIcon => self.config.tray_icon = !self.config.tray_icon,
            super::modal::SettingItem::Notifications => {
                self.config.notifications = !self.config.notifications
            }
            super::modal::SettingItem::Language => {
                self.config.language = self.config.language.next();
                i18n::set_language(self.config.language);
            }
            super::modal::SettingItem::Theme => {
                self.config.theme = self.config.theme.next();
            }
            super::modal::SettingItem::SpotifyStopOnQuit => {
                self.config.spotify.stop_on_quit = !self.config.spotify.stop_on_quit;
            }
            super::modal::SettingItem::SpotifyStartOnSpotify => {
                self.config.spotify.start_on_spotify = !self.config.spotify.start_on_spotify;
            }
            super::modal::SettingItem::SpotifyClientId => {}
            super::modal::SettingItem::SpotifyPlaybackMode => {
                self.set_spotify_playback_mode(self.config.spotify.playback_mode.next());
            }
            super::modal::SettingItem::SpotifyCrossfade => {
                if self.config.spotify.playback_mode != crate::config::SpotifyPlaybackMode::Native {
                    return;
                }
                self.config.spotify_crossfade_next();
                self.sync_native_crossfade();
            }
            super::modal::SettingItem::SpotifyRadioMode => {
                self.config.spotify.radio_enabled = !self.config.spotify.radio_enabled;
            }
            super::modal::SettingItem::YoutubeRadioMode => {
                self.config.youtube_radio_mode = !self.config.youtube_radio_mode
            }
            super::modal::SettingItem::YoutubeSponsorblock => {
                self.config.youtube_sponsorblock = !self.config.youtube_sponsorblock
            }
            super::modal::SettingItem::YoutubeCookiesPath => {}
            super::modal::SettingItem::YoutubeCookiesValidate => {}
            super::modal::SettingItem::ReplayOnboarding => {}
            super::modal::SettingItem::OpenLogs => {}
            super::modal::SettingItem::AutoUpdate => {
                self.config.auto_update = !self.config.auto_update
            }
            super::modal::SettingItem::DiscordRpc => {
                self.config.discord_rpc = !self.config.discord_rpc
            }
        }
        self.save_config();
        if let Some(ref tx) = self.windows_tx {
            let _ = tx.send(self.config.clone());
        }
    }

    async fn on_key_modal_spotify(&mut self, key: KeyCode) {
        if !matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) {
            self.on_key_spotify_auth(key);
            return;
        }

        if self.spotify_remote_blocked() {
            match key {
                KeyCode::Esc => {
                    self.show_help = false;
                    self.should_quit = true;
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    self.open_settings_at(SettingItem::SpotifyPlaybackMode);
                }
                _ => {}
            }
            return;
        }

        {
            use super::modal::SpotifySubTab;
            if key == KeyCode::Char(' ') && !matches!(self.spotify.sub_tab, SpotifySubTab::Search) {
                self.toggle_spotify_playback().await;
                return;
            }
        }

        match key {
            KeyCode::Esc => {
                self.show_help = false;
                use super::modal::SpotifySubTab;
                if self.spotify.sub_tab == SpotifySubTab::Playlists
                    && self.spotify.open_playlist.is_some()
                {
                    self.spotify.open_playlist = None;
                } else if self.spotify.sub_tab == SpotifySubTab::Albums
                    && self.spotify.open_album.is_some()
                {
                    self.spotify.open_album = None;
                    self.spotify.album_tracks_selected = 0;
                    self.spotify.album_tracks_scroll_offset = 0;
                } else if !self.spotify.search_query.is_empty() {
                    self.spotify.search_query.clear();
                    self.spotify.search_results.clear();
                    self.spotify.search_selected = 0;
                    self.spotify.search_scroll_offset = 0;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Left | KeyCode::Right => {
                let tabs = [
                    SpotifySubTab::Search,
                    SpotifySubTab::Liked,
                    SpotifySubTab::Playlists,
                    SpotifySubTab::TopTracks,
                    SpotifySubTab::Recent,
                    SpotifySubTab::Albums,
                ];
                let current = tabs
                    .iter()
                    .position(|t| *t == self.spotify.sub_tab)
                    .unwrap_or(0);
                let next = if key == KeyCode::Right {
                    (current + 1) % tabs.len()
                } else {
                    (current + tabs.len() - 1) % tabs.len()
                };
                self.switch_spotify_sub_tab(tabs[next]);
            }
            _ => match self.spotify.sub_tab {
                SpotifySubTab::Search => self.on_key_spotify_search(key).await,
                SpotifySubTab::Liked => self.on_key_spotify_liked(key).await,
                SpotifySubTab::Playlists => self.on_key_spotify_playlists(key).await,
                SpotifySubTab::TopTracks => self.on_key_spotify_top_tracks(key).await,
                SpotifySubTab::Recent => self.on_key_spotify_recent_tracks(key).await,
                SpotifySubTab::Albums => self.on_key_spotify_albums(key).await,
            },
        }
    }

    fn switch_spotify_sub_tab(&mut self, tab: SpotifySubTab) {
        self.spotify.sub_tab = tab;
        match self.spotify.sub_tab {
            SpotifySubTab::Liked
                if self.spotify.liked_tracks.is_empty() && !self.spotify.liked_loading =>
            {
                self.fetch_liked_tracks();
            }
            SpotifySubTab::Playlists
                if self.spotify.playlists.is_empty() && !self.spotify.playlists_loading =>
            {
                self.fetch_playlists();
            }
            SpotifySubTab::TopTracks
                if self.spotify.top_tracks.is_empty() && !self.spotify.top_tracks_loading =>
            {
                self.fetch_top_tracks();
            }
            SpotifySubTab::Recent
                if self.spotify.recent_tracks.is_empty() && !self.spotify.recent_tracks_loading =>
            {
                self.fetch_recent_tracks();
            }
            SpotifySubTab::Albums
                if self.spotify.albums.is_empty() && !self.spotify.albums_loading =>
            {
                self.fetch_albums();
            }
            _ => {}
        }
    }

    fn on_key_spotify_auth(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter if !matches!(self.spotify.status, SpotifyAuthStatus::Connecting) => {
                self.start_oauth_flow();
            }
            KeyCode::Char('o') | KeyCode::Char('O')
                if matches!(self.spotify.status, SpotifyAuthStatus::Idle) =>
            {
                self.open_settings_at(SettingItem::SpotifyClientId);
            }
            KeyCode::Esc => {
                self.show_help = false;
                if matches!(self.spotify.status, SpotifyAuthStatus::Connecting) {
                    abort_task(&mut self.spotify.auth_task);
                    self.spotify.auth_rx = None;
                    self.spotify.status = SpotifyAuthStatus::Idle;
                } else {
                    self.should_quit = true;
                }
            }
            _ => {}
        }
    }

    async fn toggle_radio_pause(&mut self) {
        match self.player.state().status {
            PlayerStatus::Playing => {
                self.player.send(PlayerCommand::Pause).await;
            }
            PlayerStatus::Paused => {
                self.player.send(PlayerCommand::Resume).await;
            }
            _ => {}
        }
    }

    fn open_settings_at(&mut self, item: SettingItem) {
        self.show_search_modal = true;
        self.modal_mode = SearchMode::Settings;
        self.settings_selected =
            settings_items(self.config.duck_enabled, self.config.screensaver_secs > 0)
                .iter()
                .position(|candidate| *candidate == item)
                .unwrap_or(0);
        self.settings_scroll_offset = 0;
        self.keep_settings_visible();
    }

    fn open_spotify_device_picker(&mut self) {
        if self.config.spotify.playback_mode == crate::config::SpotifyPlaybackMode::Native {
            self.notify(
                crate::app::NoticeSeverity::Info,
                t("modal.spotify.devices_native_hint"),
                5,
            );
            return;
        }
        if self.spotify.devices.is_empty() {
            if !self.spotify.devices_loading {
                self.fetch_spotify_devices();
            }
            return;
        }
        self.spotify.device_picker_selected = self
            .spotify
            .devices
            .iter()
            .position(|d| d.id.as_deref() == self.spotify.active_device_id.as_deref())
            .unwrap_or(0);
        self.spotify.device_picker_open = true;
    }

    async fn on_key_device_picker(&mut self, key: KeyCode) {
        let len = self.spotify.devices.len();
        match key {
            KeyCode::Esc => {
                self.spotify.device_picker_open = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.spotify.device_picker_selected =
                    cycle_prev(self.spotify.device_picker_selected, len);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.spotify.device_picker_selected =
                    cycle_next(self.spotify.device_picker_selected, len);
            }
            KeyCode::Enter => {
                let Some(device) = self
                    .spotify
                    .devices
                    .get(self.spotify.device_picker_selected)
                else {
                    return;
                };
                let Some(id) = device.id.clone() else {
                    return;
                };
                let name = device.name.clone();
                self.spotify.device_picker_open = false;
                self.transfer_to_spotify_device(id.clone()).await;
                if self.spotify.active_device_id.as_deref() == Some(id.as_str()) {
                    self.notify_info(t("notice.spotify_device_switched").replace("{}", &name));
                }
            }
            _ => {}
        }
    }

    async fn activate_spotify_search_selected(&mut self) {
        if let Some(track) = self
            .spotify
            .search_results
            .get(self.spotify.search_selected)
            .cloned()
        {
            self.notify(
                crate::app::NoticeSeverity::Info,
                t("notice.spotify_radio_stopped"),
                5,
            );
            let sel = self.spotify.search_selected;
            let queue = self.spotify.search_results[sel.saturating_add(1)..].to_vec();
            self.play_spotify_track_with_queue(track, queue).await;
        }
    }

    async fn activate_spotify_liked_selected(&mut self) {
        let sel = self.spotify.liked_selected;
        if sel < self.spotify.liked_tracks.len() {
            let track = self.spotify.liked_tracks[sel].clone();
            let queue = self.spotify.liked_tracks[sel + 1..].to_vec();
            self.play_spotify_track_with_queue(track, queue).await;
        }
    }

    async fn activate_spotify_playlist_selected(&mut self) {
        let sel = self.spotify.playlists_selected;
        if let Some(pl) = self.spotify.playlists.get(sel).cloned() {
            let id = pl.id.clone();
            self.spotify.open_playlist = Some(pl);
            self.fetch_playlist_tracks(id);
        }
    }

    async fn activate_spotify_playlist_track_selected(&mut self) {
        let sel = self.spotify.playlist_tracks_selected;
        if sel < self.spotify.playlist_tracks.len() {
            let track = self.spotify.playlist_tracks[sel].clone();
            let queue = self.spotify.playlist_tracks[sel + 1..].to_vec();
            self.play_spotify_track_with_queue(track, queue).await;
        }
    }

    async fn activate_spotify_top_track_selected(&mut self) {
        let sel = self.spotify.top_tracks_selected;
        if sel < self.spotify.top_tracks.len() {
            let track = self.spotify.top_tracks[sel].clone();
            let queue = self.spotify.top_tracks[sel + 1..].to_vec();
            self.play_spotify_track_with_queue(track, queue).await;
        }
    }

    async fn activate_spotify_recent_track_selected(&mut self) {
        let sel = self.spotify.recent_tracks_selected;
        if sel < self.spotify.recent_tracks.len() {
            let track = self.spotify.recent_tracks[sel].clone();
            let queue = self.spotify.recent_tracks[sel + 1..].to_vec();
            self.play_spotify_track_with_queue(track, queue).await;
        }
    }

    async fn activate_spotify_album_selected(&mut self) {
        let sel = self.spotify.albums_selected;
        if let Some(album) = self.spotify.albums.get(sel).cloned() {
            self.spotify.open_album = Some(album);
            self.spotify.album_tracks_selected = 0;
            self.spotify.album_tracks_scroll_offset = 0;
            self.fetch_album_tracks();
        }
    }

    async fn activate_spotify_album_track_selected(&mut self) {
        let sel = self.spotify.album_tracks_selected;
        if sel < self.spotify.album_tracks.len() {
            let track = self.spotify.album_tracks[sel].clone();
            let queue = self.spotify.album_tracks[sel + 1..].to_vec();
            self.play_spotify_track_with_queue(track, queue).await;
        }
    }

    async fn on_key_spotify_search(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if !self.spotify.search_results.is_empty() {
                    self.spotify.search_selected = cycle_prev(
                        self.spotify.search_selected,
                        self.spotify.search_results.len(),
                    );
                    self.keep_spotify_search_visible();
                }
            }
            KeyCode::Down => {
                if !self.spotify.search_results.is_empty() {
                    let last = self.spotify.search_results.len() - 1;
                    if self.spotify.search_selected >= last && self.spotify.search_has_more {
                        self.load_more_spotify_results();
                    } else {
                        self.spotify.search_selected = cycle_next(
                            self.spotify.search_selected,
                            self.spotify.search_results.len(),
                        );
                        self.keep_spotify_search_visible();
                    }
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_search_selected().await;
            }
            KeyCode::Backspace => {
                self.spotify.search_query.pop();
                self.spotify.search_selected = 0;
                self.spotify.search_scroll_offset = 0;
                self.perform_spotify_search();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.spotify.search_query.push(c);
                self.spotify.search_selected = 0;
                self.spotify.search_scroll_offset = 0;
                self.perform_spotify_search();
            }
            _ => {}
        }
    }

    async fn on_key_spotify_liked(&mut self, key: KeyCode) {
        let len = self.spotify.liked_tracks.len();
        match key {
            KeyCode::Up => {
                if self.spotify.liked_selected > 0 {
                    self.spotify.liked_selected -= 1;
                    self.keep_spotify_liked_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 {
                    let last = len - 1;
                    if self.spotify.liked_selected >= last && self.spotify.liked_has_more {
                        self.load_more_spotify_liked();
                    } else if self.spotify.liked_selected < last {
                        self.spotify.liked_selected += 1;
                        self.keep_spotify_liked_visible();
                    }
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_liked_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_spotify_playlists(&mut self, key: KeyCode) {
        if self.spotify.open_playlist.is_some() {
            self.on_key_spotify_playlist_tracks(key).await;
        } else {
            self.on_key_spotify_playlist_list(key).await;
        }
    }

    async fn on_key_spotify_playlist_list(&mut self, key: KeyCode) {
        let len = self.spotify.playlists.len();
        match key {
            KeyCode::Up => {
                if self.spotify.playlists_selected > 0 {
                    self.spotify.playlists_selected -= 1;
                    self.keep_spotify_playlists_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 {
                    let last = len - 1;
                    if self.spotify.playlists_selected >= last && self.spotify.playlists_has_more {
                        self.load_more_spotify_playlists();
                    } else if self.spotify.playlists_selected < last {
                        self.spotify.playlists_selected += 1;
                        self.keep_spotify_playlists_visible();
                    }
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_playlist_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_spotify_playlist_tracks(&mut self, key: KeyCode) {
        let len = self.spotify.playlist_tracks.len();
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                self.spotify.open_playlist = None;
            }
            KeyCode::Up => {
                if self.spotify.playlist_tracks_selected > 0 {
                    self.spotify.playlist_tracks_selected -= 1;
                    self.keep_spotify_playlist_tracks_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 {
                    let last = len - 1;
                    if self.spotify.playlist_tracks_selected >= last
                        && self.spotify.playlist_tracks_has_more
                    {
                        self.load_more_spotify_playlist_tracks();
                    } else if self.spotify.playlist_tracks_selected < last {
                        self.spotify.playlist_tracks_selected += 1;
                        self.keep_spotify_playlist_tracks_visible();
                    }
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_playlist_track_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_spotify_top_tracks(&mut self, key: KeyCode) {
        let len = self.spotify.top_tracks.len();
        match key {
            KeyCode::Up => {
                if self.spotify.top_tracks_selected > 0 {
                    self.spotify.top_tracks_selected -= 1;
                    self.keep_spotify_top_tracks_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.spotify.top_tracks_selected < len - 1 {
                    self.spotify.top_tracks_selected += 1;
                    self.keep_spotify_top_tracks_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_top_track_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_spotify_recent_tracks(&mut self, key: KeyCode) {
        let len = self.spotify.recent_tracks.len();
        match key {
            KeyCode::Up => {
                if self.spotify.recent_tracks_selected > 0 {
                    self.spotify.recent_tracks_selected -= 1;
                    self.keep_spotify_recent_tracks_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.spotify.recent_tracks_selected < len - 1 {
                    self.spotify.recent_tracks_selected += 1;
                    self.keep_spotify_recent_tracks_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_recent_track_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_spotify_albums(&mut self, key: KeyCode) {
        if self.spotify.open_album.is_some() {
            self.on_key_spotify_album_tracks(key).await;
        } else {
            self.on_key_spotify_album_list(key).await;
        }
    }

    async fn on_key_spotify_album_list(&mut self, key: KeyCode) {
        let len = self.spotify.albums.len();
        match key {
            KeyCode::Up => {
                if self.spotify.albums_selected > 0 {
                    self.spotify.albums_selected -= 1;
                    self.keep_spotify_albums_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 {
                    let last = len - 1;
                    if self.spotify.albums_selected >= last && self.spotify.albums_has_more {
                        self.load_more_spotify_albums();
                    } else if self.spotify.albums_selected < last {
                        self.spotify.albums_selected += 1;
                        self.keep_spotify_albums_visible();
                    }
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_album_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_spotify_album_tracks(&mut self, key: KeyCode) {
        let len = self.spotify.album_tracks.len();
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                self.spotify.open_album = None;
                self.spotify.album_tracks_selected = 0;
                self.spotify.album_tracks_scroll_offset = 0;
            }
            KeyCode::Up => {
                if self.spotify.album_tracks_selected > 0 {
                    self.spotify.album_tracks_selected -= 1;
                    self.keep_spotify_album_tracks_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.spotify.album_tracks_selected < len - 1 {
                    self.spotify.album_tracks_selected += 1;
                    self.keep_spotify_album_tracks_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_spotify_album_track_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_modal_youtube(&mut self, key: KeyCode) {
        if key == KeyCode::Char(' ') && !matches!(self.youtube.sub_tab, YoutubeSubTab::Search) {
            self.toggle_radio_pause().await;
            return;
        }
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                if self.youtube.sub_tab == YoutubeSubTab::Playlists
                    && self.youtube.open_playlist.is_some()
                {
                    self.close_youtube_playlist();
                } else if self.youtube.sub_tab == YoutubeSubTab::Search
                    && (!self.youtube.query.is_empty() || !self.youtube.results.is_empty())
                {
                    self.youtube.query.clear();
                    self.youtube.results.clear();
                    self.youtube.selected = 0;
                    self.youtube.scroll_offset = 0;
                    self.youtube.loading = false;
                    self.youtube.search_pending_until = None;
                    abort_task(&mut self.youtube.search_task);
                    self.youtube.search_rx = None;
                    if crate::integrations::youtube::runtime_installed() {
                        self.youtube.status = super::YoutubeStatus::Ready;
                    } else {
                        self.youtube.status = super::YoutubeStatus::Idle;
                    }
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Left | KeyCode::Right => {
                let tabs = [
                    YoutubeSubTab::Search,
                    YoutubeSubTab::Bookmarks,
                    YoutubeSubTab::Liked,
                    YoutubeSubTab::Playlists,
                ];
                let current = tabs
                    .iter()
                    .position(|t| *t == self.youtube.sub_tab)
                    .unwrap_or(0);
                let next = if key == KeyCode::Right {
                    (current + 1) % tabs.len()
                } else {
                    (current + tabs.len() - 1) % tabs.len()
                };
                self.switch_youtube_sub_tab(tabs[next]);
            }
            _ => {
                if matches!(
                    self.youtube.sub_tab,
                    YoutubeSubTab::Liked | YoutubeSubTab::Playlists
                ) && (self.config.youtube.cookies_path.is_none() || self.youtube.cookies_invalid)
                    && matches!(key, KeyCode::Char('o') | KeyCode::Char('O'))
                {
                    self.open_settings_at(SettingItem::YoutubeCookiesPath);
                    return;
                }
                match self.youtube.sub_tab {
                    YoutubeSubTab::Search => self.on_key_youtube_search(key).await,
                    YoutubeSubTab::Bookmarks => self.on_key_youtube_bookmarks(key).await,
                    YoutubeSubTab::Liked => self.on_key_youtube_liked(key).await,
                    YoutubeSubTab::Playlists => self.on_key_youtube_playlists(key).await,
                }
            }
        }
    }

    fn switch_youtube_sub_tab(&mut self, tab: YoutubeSubTab) {
        self.youtube.sub_tab = tab;
        match self.youtube.sub_tab {
            YoutubeSubTab::Liked
                if self.youtube.liked_videos.is_empty() && !self.youtube.liked_loading =>
            {
                self.fetch_youtube_liked();
            }
            YoutubeSubTab::Playlists
                if self.youtube.playlists.is_empty() && !self.youtube.playlists_loading =>
            {
                self.fetch_youtube_playlists();
            }
            _ => {}
        }
    }

    async fn on_key_youtube_search(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if !self.youtube.results.is_empty() {
                    self.youtube.selected =
                        cycle_prev(self.youtube.selected, self.youtube.results.len());
                    self.keep_youtube_search_visible();
                }
            }
            KeyCode::Down => {
                if !self.youtube.results.is_empty() {
                    self.youtube.selected =
                        cycle_next(self.youtube.selected, self.youtube.results.len());
                    self.keep_youtube_search_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_youtube_selected().await;
            }
            KeyCode::Backspace => {
                self.youtube.query.pop();
                self.youtube.selected = 0;
                self.youtube.scroll_offset = 0;
                self.perform_youtube_search();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.youtube.query.push(c);
                self.youtube.selected = 0;
                self.youtube.scroll_offset = 0;
                self.perform_youtube_search();
            }
            _ => {}
        }
    }

    async fn on_key_youtube_bookmarks(&mut self, key: KeyCode) {
        let len = self.youtube.bookmarks.len();
        match key {
            KeyCode::Up => {
                if self.youtube.bookmarks_selected > 0 {
                    self.youtube.bookmarks_selected -= 1;
                    self.keep_youtube_bookmarks_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.youtube.bookmarks_selected < len - 1 {
                    self.youtube.bookmarks_selected += 1;
                    self.keep_youtube_bookmarks_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_youtube_bookmark_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_youtube_liked(&mut self, key: KeyCode) {
        let len = self.youtube.liked_videos.len();
        match key {
            KeyCode::Up => {
                if self.youtube.liked_selected > 0 {
                    self.youtube.liked_selected -= 1;
                    self.keep_youtube_liked_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.youtube.liked_selected < len - 1 {
                    self.youtube.liked_selected += 1;
                    self.keep_youtube_liked_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_youtube_liked_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_youtube_playlists(&mut self, key: KeyCode) {
        if self.youtube.open_playlist.is_some() {
            self.on_key_youtube_playlist_videos(key).await;
        } else {
            self.on_key_youtube_playlist_list(key).await;
        }
    }

    async fn on_key_youtube_playlist_list(&mut self, key: KeyCode) {
        let len = self.youtube.playlists.len();
        match key {
            KeyCode::Up => {
                if self.youtube.playlists_selected > 0 {
                    self.youtube.playlists_selected -= 1;
                    self.keep_youtube_playlists_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.youtube.playlists_selected < len - 1 {
                    self.youtube.playlists_selected += 1;
                    self.keep_youtube_playlists_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_youtube_playlist_selected().await;
            }
            _ => {}
        }
    }

    async fn on_key_youtube_playlist_videos(&mut self, key: KeyCode) {
        let len = self.youtube.playlist_videos.len();
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                self.close_youtube_playlist();
            }
            KeyCode::Up => {
                if self.youtube.playlist_videos_selected > 0 {
                    self.youtube.playlist_videos_selected -= 1;
                    self.keep_youtube_playlist_videos_visible();
                }
            }
            KeyCode::Down => {
                if len > 0 && self.youtube.playlist_videos_selected < len - 1 {
                    self.youtube.playlist_videos_selected += 1;
                    self.keep_youtube_playlist_videos_visible();
                }
            }
            KeyCode::Enter => {
                self.activate_youtube_playlist_video_selected().await;
            }
            _ => {}
        }
    }

    async fn activate_youtube_selected(&mut self) {
        if !self.youtube.results.is_empty() {
            self.play_youtube_from_context(
                crate::app::youtube_state::YoutubePlaybackContext::SearchResults,
                self.youtube.selected,
            );
        } else if !self.youtube.query.trim().is_empty() {
            self.start_youtube_search_now();
        } else {
            self.ensure_youtube_ready();
        }
    }

    async fn activate_youtube_liked_selected(&mut self) {
        self.play_youtube_from_context(
            crate::app::youtube_state::YoutubePlaybackContext::LikedVideos,
            self.youtube.liked_selected,
        );
    }

    async fn activate_youtube_bookmark_selected(&mut self) {
        self.play_youtube_from_context(
            crate::app::youtube_state::YoutubePlaybackContext::Bookmarks,
            self.youtube.bookmarks_selected,
        );
    }

    async fn activate_youtube_playlist_selected(&mut self) {
        let sel = self.youtube.playlists_selected;
        if let Some(playlist) = self.youtube.playlists.get(sel).cloned() {
            self.fetch_youtube_playlist_videos(playlist);
        }
    }

    async fn activate_youtube_playlist_video_selected(&mut self) {
        self.play_youtube_from_context(
            crate::app::youtube_state::YoutubePlaybackContext::PlaylistVideos,
            self.youtube.playlist_videos_selected,
        );
    }

    async fn on_key_recent(&mut self, key: KeyCode) {
        let titles = self.player.state().recent_titles;
        let len = titles.len();
        if len == 0 {
            self.focus = AppFocus::Stations;
            return;
        }
        self.recent_selected = self.recent_selected.min(len - 1);

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.recent_selected = self.recent_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.recent_selected + 1 < len {
                    self.recent_selected += 1;
                }
            }
            KeyCode::Enter => {
                let title = titles[self.recent_selected].clone();
                let state = self.player.state();
                let key_str = state
                    .station
                    .as_ref()
                    .map(|s| s.key.as_str())
                    .unwrap_or("unknown");
                match library::save_track(&title, key_str) {
                    library::SaveResult::Saved => {
                        self.saved_tracks = library::load_saved_tracks(key_str);
                        self.notify_info(format!("{} {title}", t("notice.saved")));
                    }
                    library::SaveResult::AlreadySaved => {
                        self.notify_warning(format!("{} {title}", t("notice.already_saved")));
                    }
                }
            }
            KeyCode::Char('p') => {
                let state = self.player.state();
                if state.preview_title.is_some() || state.preview_searching {
                    self.player.send(PlayerCommand::StopPreview).await;
                    self.player
                        .send(PlayerCommand::SetPreviewSearching(false))
                        .await;
                } else if !titles.is_empty() {
                    let raw = titles[self.recent_selected].clone();
                    let preview_id = self.next_preview_id();
                    let cmd_tx = self.player.clone_sender();
                    let _ = cmd_tx.send(PlayerCommand::SetPreviewSearching(true)).await;
                    let _ = cmd_tx
                        .send(PlayerCommand::SetPreviewLoadingTrack(Some(raw.clone())))
                        .await;
                    tokio::spawn(async move {
                        match deezer_preview(&raw).await {
                            Some((url, title)) => {
                                let _ = cmd_tx
                                    .send(PlayerCommand::SetPreviewLoadingTrack(None))
                                    .await;
                                if cmd_tx
                                    .send(PlayerCommand::PlayPreview {
                                        url,
                                        title,
                                        raw_track: raw,
                                        preview_id: Some(preview_id),
                                        start_at_secs: 0.0,
                                    })
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                tokio::time::sleep(std::time::Duration::from_secs(35)).await;
                                let _ = cmd_tx
                                    .send(PlayerCommand::StopPreviewIfCurrent(preview_id))
                                    .await;
                            }
                            None => {
                                tracing::warn!("Deezer: no result for '{raw}'");
                                let _ = cmd_tx
                                    .send(PlayerCommand::SetPreviewLoadingTrack(None))
                                    .await;
                                let _ =
                                    cmd_tx.send(PlayerCommand::SetPreviewSearching(false)).await;
                                let _ = cmd_tx
                                    .send(PlayerCommand::MarkPreviewUnavailable(raw))
                                    .await;
                            }
                        }
                    });
                }
            }
            KeyCode::Esc => {
                self.focus = AppFocus::Stations;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::youtube::YoutubeVideo;

    fn youtube_video(id: &str) -> YoutubeVideo {
        YoutubeVideo {
            id: id.to_string(),
            title: format!("Video {id}"),
            channel: "Channel".to_string(),
            duration_secs: 120,
            watch_url: format!("https://youtube.test/watch?v={id}"),
            thumbnail: None,
            is_live: false,
        }
    }

    #[test]
    fn setting_index_at_visual_row_ignores_headers() {
        let items = settings_items(false, false);

        assert_eq!(setting_index_at_visual_row(&items, 0), None);
        assert_eq!(setting_index_at_visual_row(&items, 6), None);
    }

    #[test]
    fn setting_index_at_visual_row_returns_item_index() {
        let items = settings_items(false, false);

        assert_eq!(setting_index_at_visual_row(&items, 1), Some(0));
        assert_eq!(setting_index_at_visual_row(&items, 7), Some(5));
    }

    #[test]
    fn settings_visual_row_count_includes_headers() {
        let items = settings_items(false, false);
        let mut groups: Vec<&str> = items.iter().map(|item| item.group_key()).collect();
        groups.dedup();

        assert_eq!(
            settings_visual_row_count(&items),
            items.len() + groups.len()
        );
    }

    #[tokio::test]
    async fn paste_routes_to_active_text_input() {
        let mut app = App::new().await;

        app.editing_client_id = true;
        app.on_paste("spotify-client\n".to_string());
        assert_eq!(app.client_id_input, "spotify-client");

        app.editing_client_id = false;
        app.editing_cookies_path = true;
        app.cookies_path_error = Some("old error".to_string());
        app.on_paste("/tmp/cookies.txt\n".to_string());
        assert_eq!(app.cookies_path_input, "/tmp/cookies.txt");
        assert_eq!(app.cookies_path_error, None);

        app.editing_cookies_path = false;
        app.show_search_modal = true;
        app.modal_mode = SearchMode::Spotify;
        app.spotify.sub_tab = SpotifySubTab::Search;
        app.on_paste("daft punk".to_string());
        assert_eq!(app.spotify.search_query, "daft punk");

        app.modal_mode = SearchMode::Youtube;
        app.youtube.sub_tab = YoutubeSubTab::Search;
        app.on_paste("lofi radio".to_string());
        assert_eq!(app.youtube.query, "lofi radio");
    }

    #[tokio::test]
    async fn active_copy_text_uses_visible_or_editing_input() {
        let mut app = App::new().await;

        app.show_search_modal = true;
        app.modal_mode = SearchMode::Spotify;
        app.spotify.sub_tab = SpotifySubTab::Search;
        app.spotify.search_query = "boards of canada".to_string();
        app.search_query = "radio query".to_string();
        assert_eq!(app.active_copy_text(), Some("boards of canada".to_string()));

        app.editing_client_id = true;
        app.client_id_input = "client-id".to_string();
        assert_eq!(app.active_copy_text(), Some("client-id".to_string()));

        app.editing_client_id = false;
        app.modal_mode = SearchMode::Name;
        app.radio_sub_tab = RadioSubTab::Favorites;
        assert_eq!(app.active_copy_text(), None);
    }

    #[tokio::test]
    async fn first_click_while_screensaver_is_active_only_wakes_it() {
        let mut app = App::new().await;
        app.terminal_area = ratatui::layout::Rect::new(0, 0, 100, 40);
        app.show_search_modal = true;
        app.modal_mode = SearchMode::Youtube;
        app.config.screensaver_secs = 1;
        app.config.screensaver_clock = true;
        app.last_activity = Instant::now() - std::time::Duration::from_secs(2);
        app.youtube.results = vec![youtube_video("one"), youtube_video("two")];
        app.youtube.selected = 0;

        let list_area =
            youtube_search_list_area(app.terminal_area).expect("youtube list should render");
        app.on_click(list_area.x, list_area.y + 2).await;

        assert!(!app.screensaver_active());
        assert_eq!(app.youtube.selected, 0);
        assert!(app.youtube.resolve_task.is_none());
    }
}
