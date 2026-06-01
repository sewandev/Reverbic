use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::app::{IntegrationView, SpotifyAuthStatus, SpotifyField};
use crate::i18n::t;
use crate::ui::theme;

use super::helpers::{key, sep_s, spin_frame};
use super::{BG, SearchModalWidget};

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_integrations_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let lx = content_x + 2;
        let lw = content_w.saturating_sub(2);
        match self.integration_view {
            IntegrationView::ServiceList       => self.render_integ_service_list(area, lx, lw, buf),
            IntegrationView::SpotifyDetail     => self.render_integ_spotify_detail(area, lx, lw, buf),
            IntegrationView::SpotifyUserPass   => self.render_integ_spotify_userpass(area, content_x, content_w, lx, lw, buf),
            IntegrationView::SpotifyWebBrowser => self.render_integ_spotify_web(area, lx, lw, buf),
        }
    }

    pub(super) fn render_integ_service_list(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 1;
        if y >= area.bottom() { return; }
        Paragraph::new(Span::styled(
            format!("── {} ", t("integrations.services")),
            Style::default().fg(theme::MUTED),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 2;
        if y >= area.bottom() { return; }

        let active = self.integration_selected == 0;
        let status_str = match self.spotify_saved {
            Some(name) => format!("  [{}]", name),
            None       => String::new(),
        };
        let max_name = lw.saturating_sub(3 + status_str.chars().count() as u16) as usize;
        let (prefix, name_st, meta_st) = if active {
            ("▶  ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD), Style::default().fg(theme::MUTED))
        } else {
            ("   ", Style::default().fg(theme::HIGHLIGHT), Style::default().fg(theme::MUTED))
        };
        let name = "Spotify";
        let name_display: String = if name.len() > max_name { name.chars().take(max_name).collect() } else { name.to_string() };
        Paragraph::new(Line::from(vec![
            Span::styled(prefix,       name_st),
            Span::styled(name_display, name_st),
            Span::styled(status_str,   meta_st),
        ]))
        .render(Rect::new(lx, y, lw, 1), buf);
    }

    pub(super) fn render_integ_spotify_detail(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 1;
        if y >= area.bottom() { return; }

        let header = Line::from(vec![
            Span::styled("← ", Style::default().fg(theme::MUTED)),
            Span::styled("Spotify", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]);
        Paragraph::new(header).render(Rect::new(lx, y, lw, 1), buf);
        y += 2;

        if matches!(self.spotify_status, SpotifyAuthStatus::LoggedIn) {
            if y >= area.bottom() { return; }
            Paragraph::new(Span::styled(t("integrations.spotify.logged_in"), Style::default().fg(theme::MUTED)))
                .render(Rect::new(lx, y, lw, 1), buf);
            y += 1;
            if y >= area.bottom() { return; }
            let name = self.spotify_saved.unwrap_or("Spotify");
            Paragraph::new(Line::from(vec![
                Span::styled("▶  ", Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD)),
                Span::styled(name,   Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD)),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
            y += 2;
            if y >= area.bottom() { return; }
            Paragraph::new(Line::from(vec![
                key("[D]"),
                sep_s(format!(" {}", t("integrations.spotify.logout_action"))),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
            return;
        }

        if let Some(name) = self.spotify_saved {
            if y < area.bottom() {
                Paragraph::new(Line::from(vec![
                    Span::styled(t("integrations.spotify.saved"), Style::default().fg(theme::MUTED)),
                    Span::styled(format!(" {name}"), Style::default().fg(theme::DIM)),
                ]))
                .render(Rect::new(lx, y, lw, 1), buf);
                y += 1;
            }
        }

        let methods = [
            t("integrations.spotify.method.userpass"),
            t("integrations.spotify.method.browser"),
        ];
        for (i, label) in methods.iter().enumerate() {
            if y >= area.bottom() { break; }
            let active = i == self.spotify_auth_selected;
            let (prefix, st) = if active {
                ("▶  ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
            } else {
                ("   ", Style::default().fg(theme::HIGHLIGHT))
            };
            Paragraph::new(Line::from(vec![
                Span::styled(prefix,        st),
                Span::styled(label.clone(), st),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
            y += 1;
        }
    }

    pub(super) fn render_integ_spotify_userpass(
        &self, area: Rect, cx: u16, cw: u16, lx: u16, lw: u16, buf: &mut Buffer,
    ) {
        let tx = cx + 2;
        let tw = cw.saturating_sub(2);

        if matches!(self.spotify_status, SpotifyAuthStatus::Connecting) {
            let y = area.y + area.height / 2;
            Paragraph::new(Line::from(vec![
                Span::styled(spin_frame(), Style::default().fg(theme::ACCENT)),
                Span::styled(format!("  {}", t("integrations.spotify.connecting")), Style::default().fg(theme::MUTED)),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
            return;
        }

        let mut y = area.y;
        let header = Line::from(vec![
            Span::styled("← ", Style::default().fg(theme::MUTED)),
            Span::styled(t("integrations.spotify.method.userpass"), Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]);
        if let SpotifyAuthStatus::Error(msg) = self.spotify_status {
            let max = lw as usize;
            let display: String = if msg.chars().count() > max {
                msg.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
            } else {
                msg.clone()
            };
            Paragraph::new(Span::styled(display, Style::default().fg(theme::WARNING)))
                .render(Rect::new(lx, y, lw, 1), buf);
        } else {
            Paragraph::new(header).render(Rect::new(lx, y, lw, 1), buf);
        }
        y += 2;

        let user_active = matches!(self.spotify_field, SpotifyField::Username);
        let pass_active = !user_active;

        if y < area.bottom() {
            Paragraph::new(Span::styled(
                t("integrations.spotify.username"),
                Style::default().fg(if user_active { theme::ACCENT } else { theme::MUTED }),
            ))
            .render(Rect::new(lx, y, lw, 1), buf);
            y += 1;
        }
        if y < area.bottom() {
            buf[(cx, y)].set_symbol("┃").set_fg(if user_active { theme::ACCENT } else { theme::DIM }).set_bg(BG);
            let max_w = tw.saturating_sub(1) as usize;
            let visible: String = if self.spotify_username.chars().count() > max_w {
                self.spotify_username.chars().rev().take(max_w).collect::<String>().chars().rev().collect()
            } else {
                self.spotify_username.to_owned()
            };
            let mut spans = vec![Span::styled(visible, Style::default().fg(theme::HIGHLIGHT))];
            if user_active {
                spans.push(Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)));
            }
            Paragraph::new(Line::from(spans)).render(Rect::new(tx, y, tw, 1), buf);
            y += 2;
        }

        if y < area.bottom() {
            Paragraph::new(Span::styled(
                t("integrations.spotify.password"),
                Style::default().fg(if pass_active { theme::ACCENT } else { theme::MUTED }),
            ))
            .render(Rect::new(lx, y, lw, 1), buf);
            y += 1;
        }
        if y < area.bottom() {
            buf[(cx, y)].set_symbol("┃").set_fg(if pass_active { theme::ACCENT } else { theme::DIM }).set_bg(BG);
            let dots = self.spotify_pw_len.min(tw.saturating_sub(1) as usize);
            let mut spans = vec![Span::styled("•".repeat(dots), Style::default().fg(theme::HIGHLIGHT))];
            if pass_active {
                spans.push(Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)));
            }
            Paragraph::new(Line::from(spans)).render(Rect::new(tx, y, tw, 1), buf);
        }
    }

    pub(super) fn render_integ_spotify_web(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 1;
        if y >= area.bottom() { return; }

        Paragraph::new(Line::from(vec![
            Span::styled("← ", Style::default().fg(theme::MUTED)),
            Span::styled(t("integrations.spotify.method.browser"), Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 2;

        if matches!(self.spotify_status, SpotifyAuthStatus::Connecting) {
            if y < area.bottom() {
                Paragraph::new(Line::from(vec![
                    Span::styled(spin_frame(), Style::default().fg(theme::ACCENT)),
                    Span::styled(format!("  {}", t("integrations.spotify.web.waiting")), Style::default().fg(theme::MUTED)),
                ]))
                .render(Rect::new(lx, y, lw, 1), buf);
                y += 1;
            }
            if y < area.bottom() {
                Paragraph::new(Span::styled(t("integrations.spotify.web.waiting2"), Style::default().fg(theme::DIM)))
                    .render(Rect::new(lx, y, lw, 1), buf);
            }
            return;
        }

        if let SpotifyAuthStatus::Error(msg) = self.spotify_status {
            let max = lw as usize;
            let display: String = if msg.chars().count() > max {
                msg.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
            } else {
                msg.clone()
            };
            if y < area.bottom() {
                Paragraph::new(Span::styled(display, Style::default().fg(theme::WARNING)))
                    .render(Rect::new(lx, y, lw, 1), buf);
                y += 1;
            }
            if y < area.bottom() {
                Paragraph::new(Span::styled(t("integrations.spotify.web.retry"), Style::default().fg(theme::DIM)))
                    .render(Rect::new(lx, y, lw, 1), buf);
            }
            return;
        }

        let lines = [
            t("integrations.spotify.web.line1"),
            t("integrations.spotify.web.line2"),
            t("integrations.spotify.web.line3"),
        ];
        for line in &lines {
            if y >= area.bottom() { break; }
            Paragraph::new(Span::styled(line.clone(), Style::default().fg(theme::MUTED)))
                .render(Rect::new(lx, y, lw, 1), buf);
            y += 1;
        }
        y += 1;
        if y < area.bottom() {
            Paragraph::new(Line::from(vec![
                key("[↵]"),
                sep_s(format!(" {}", t("integrations.spotify.web.open_short"))),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
        }
    }
}
