use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::App;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum NoticeSeverity {
    #[default]
    Info,
    Warning,
    Error,
}

const NOTICE_QUEUE_CAP: usize = 4;

impl App {
    pub fn notify(&mut self, severity: NoticeSeverity, text: String, secs: u64) {
        let active = self.save_notice.is_some()
            && self
                .notice_until
                .is_some_and(|until| Instant::now() < until);
        if active && severity <= self.save_notice_severity {
            if self.save_notice.as_deref() != Some(text.as_str())
                && self.notice_queue.len() < NOTICE_QUEUE_CAP
            {
                self.notice_queue.push_back((severity, text, secs));
            }
            return;
        }
        self.set_current_notice(severity, text, secs);
    }

    pub fn notify_info(&mut self, text: String) {
        self.notify(NoticeSeverity::Info, text, 3);
    }

    pub fn notify_warning(&mut self, text: String) {
        self.notify(NoticeSeverity::Warning, text, 6);
    }

    pub fn notify_error(&mut self, text: String) {
        self.notify(NoticeSeverity::Error, text, 8);
    }

    pub fn advance_notice_queue(&mut self) {
        match self.notice_queue.pop_front() {
            Some((severity, text, secs)) => self.set_current_notice(severity, text, secs),
            None => {
                self.save_notice = None;
                self.notice_until = None;
            }
        }
    }

    pub fn clear_notices(&mut self) {
        self.save_notice = None;
        self.notice_until = None;
        self.notice_queue.clear();
    }

    fn set_current_notice(&mut self, severity: NoticeSeverity, text: String, secs: u64) {
        self.save_notice = Some(text);
        self.save_notice_severity = severity;
        self.notice_until = Some(Instant::now() + Duration::from_secs(secs));
    }
}

pub(super) type NoticeQueue = VecDeque<(NoticeSeverity, String, u64)>;
