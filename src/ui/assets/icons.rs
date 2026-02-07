use crate::torrent::{State, TorrentInfo};

pub struct Icons;

impl Icons {
    pub const ACTIVE: char = '󰐊';
    pub const PAUSED: char = '󰏤';
    pub const FINISHED: char = '';
    pub const INITIALIZING: char = '';
    pub const PEERS: char = '';
    pub const UPLOADING: char = '󰕒';
    pub const DOWNLOADING: char = '󰇚';
    pub const ERROR: char = '';
    pub const WHICHKEY_DIVIDER: char = '';

    pub fn status(torrent_info: &TorrentInfo) -> char {
        match torrent_info.state {
            State::Paused => Self::PAUSED,
            State::Active => {
                if torrent_info.finished {
                    Self::FINISHED
                } else {
                    Self::ACTIVE
                }
            }
            State::Initializing => Self::INITIALIZING,
            State::Error => Self::ERROR,
        }
    }
}
