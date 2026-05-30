use crossterm::{
    event::{EnableBracketedPaste, EnableMouseCapture},
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
        SetTitle("Reverbic"),
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .map_err(|e| AppError::Terminal(e.to_string()))?;
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(|e| AppError::Terminal(e.to_string()))?;
    Ok(terminal)
}
pub fn restore() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
}
