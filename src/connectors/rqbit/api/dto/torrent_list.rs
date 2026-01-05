use serde::Deserialize;

use crate::torrent::InfoHash;

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Eq, Clone))]
pub struct TorrentListResponse {
    pub torrents: Vec<TorrentItemResponse>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy, serde::Serialize, PartialEq, Eq, Clone))]
pub struct TorrentItemResponse {
    pub info_hash: InfoHash,
    pub name: String,
    pub output_folder: String,
}
