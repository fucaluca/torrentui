use serde::Deserialize;

use crate::app::CurrentScreen;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum AddMagnetMode {
    #[default]
    Input,
    Connectors,
}

impl AddMagnetMode {
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
    AddMagnet(AddMagnetMode),
}

use std::str::FromStr;

impl FromStr for KeyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "torrent-list" => Ok(KeyMode::TorrentList),
            _ => Err(format!("Unknown key mode: {}", s)),
        }
    }
}

impl From<&CurrentScreen> for KeyMode {
    fn from(screen: &CurrentScreen) -> Self {
        match screen {
            CurrentScreen::AddMagnet(mode) => Self::AddMagnet(*mode),
            CurrentScreen::TorrentList => Self::TorrentList,
        }
    }
}
