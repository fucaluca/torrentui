#![allow(dead_code)] // TODO: remove this

pub mod info_hash;
pub mod source;

use std::sync::Arc;

pub use info_hash::InfoHash;
pub use source::Source;

#[derive(Debug)]
#[cfg_attr(test, derive(fake::Dummy))]
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
    pub connector_name: Arc<String>,
    pub peer_queued: u32,
    pub peer_live: u32,
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

#[derive(Debug)]
#[cfg_attr(test, derive(fake::Dummy, Clone))]
pub enum State {
    Active,
    Paused,
    Error,
}
