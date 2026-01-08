use serde::Deserialize;

use crate::torrent::{self};

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize))]
pub struct TorrentInfoResponse {
    pub finished: bool,
    pub state: TorrentStateResponse,
    pub progress_bytes: usize,
    pub uploaded_bytes: usize,
    pub total_bytes: usize,
    pub live: Option<TorrentLiveResponse>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq))]
#[serde(rename_all = "lowercase")]
pub enum TorrentStateResponse {
    Live,
    Paused,
    Error,
}

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize))]
pub struct TorrentLiveResponse {
    pub download_speed: TorrentSpeedResponse,
    pub upload_speed: TorrentSpeedResponse,
    pub time_remaining: Option<TorrentTimeRemainingResponse>,
    pub snapshot: TorrentSnapshotResponse,
}

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize))]
pub struct TorrentSpeedResponse {
    pub mbps: f32,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq))]
pub struct TorrentTimeRemainingResponse {
    pub duration: TorrentTimeRemainingDurationResponse,
}

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq))]
pub struct TorrentSnapshotResponse {
    pub peer_stats: PeerStats,
}

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq))]
pub struct PeerStats {
    pub queued: u32,
    pub live: u32,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq))]
pub struct TorrentTimeRemainingDurationResponse {
    pub secs: usize,
}

impl From<TorrentStateResponse> for torrent::State {
    fn from(value: TorrentStateResponse) -> Self {
        match value {
            TorrentStateResponse::Live => Self::Active,
            TorrentStateResponse::Paused => Self::Paused,
            TorrentStateResponse::Error => Self::Error,
        }
    }
}
