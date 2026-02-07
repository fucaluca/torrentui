use serde::Deserialize;

use crate::app::CurrentScreen;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum KeyMode {
    #[default]
    TorrentList,
    AddTorrent,
}

impl From<&CurrentScreen> for KeyMode {
    fn from(screen: &CurrentScreen) -> Self {
        use CurrentScreen::*;
        match screen {
            AddTorrent => Self::AddTorrent,
            TorrentList => Self::TorrentList,
        }
    }
}
