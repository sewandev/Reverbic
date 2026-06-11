use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum YoutubeError {
    Install(String),
    Search(String),
    Resolve(String),
    Cookies(String),
    Library(String),
}

impl Display for YoutubeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Install(msg)
            | Self::Search(msg)
            | Self::Resolve(msg)
            | Self::Cookies(msg)
            | Self::Library(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for YoutubeError {}
