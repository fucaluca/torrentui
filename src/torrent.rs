pub mod info_hash;
pub mod source;

pub use info_hash::InfoHash;
use serde::Deserialize;
pub use source::Source;

#[cfg(test)]
pub use source::Magnet;

#[derive(Debug)]
#[cfg_attr(test, derive(fake::Dummy, Clone))]
pub struct TorrentInfo {
    pub name: String,
    pub info_hash: InfoHash,
    pub output_folder: String,
    pub finished: bool,
    pub state: State,
    pub downloaded_bytes: usize,
    pub uploaded_bytes: usize,
    pub total_bytes: usize,
    pub download_speed_mpbs: f32,
    pub upload_speed_mpbs: f32,
    pub time_remaining_secs: Option<usize>,
    pub peer_live: u32,
    pub peer_seen: u32,
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Eq, Clone))]
pub struct TorrentList {
    pub torrents: Vec<TorrentItem>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq, Clone))]
pub struct TorrentItem {
    pub info_hash: InfoHash,
    pub name: String,
    pub output_folder: String,
}

#[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
#[cfg_attr(test, derive(fake::Dummy, Clone))]
pub enum State {
    Active,
    Paused,
    Initializing,
    Error,
}
