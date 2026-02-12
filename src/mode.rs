use serde::Deserialize;

use crate::app::CurrentScreen;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum AddTorrentMode {
    #[default]
    Input,
    Connectors,
}

impl AddTorrentMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Connectors => Self::Input,
            Self::Input => Self::Connectors,
        };
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum KeyMode {
    #[default]
    TorrentList,
    AddTorrent(AddTorrentMode),
}

use std::str::FromStr;

impl FromStr for KeyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TorrentList" => Ok(KeyMode::TorrentList),
            _ => Err(format!("Unknown key mode: {}", s)),
        }
    }
}

impl From<&CurrentScreen> for KeyMode {
    fn from(screen: &CurrentScreen) -> Self {
        match screen {
            CurrentScreen::AddTorrent(mode) => Self::AddTorrent(*mode),
            CurrentScreen::TorrentList => Self::TorrentList,
        }
    }
}
