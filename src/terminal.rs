use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

use crate::error::{AppError, Result};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;
pub fn init() -> Result<Tui> {
    enable_raw_mode().map_err(|e| AppError::Terminal(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        SetTitle(concat!("Reverbic v", env!("CARGO_PKG_VERSION"))),
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .map_err(|e| {
        restore();
        AppError::Terminal(e.to_string())
    })?;
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(|e| {
        restore();
        AppError::Terminal(e.to_string())
    })?;
    Ok(terminal)
}
pub fn restore() {
    let _ = execute!(
        io::stdout(),
        DisableMouseCapture,
        DisableBracketedPaste,
        LeaveAlternateScreen
    );
    let _ = disable_raw_mode();
}

pub fn set_title(title: &str) {
    let _ = execute!(io::stdout(), SetTitle(title));
}
