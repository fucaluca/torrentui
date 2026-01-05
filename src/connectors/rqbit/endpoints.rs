use crate::connectors::InfoHash;

#[derive(Default)]
pub struct Endpoints;

impl Endpoints {
    pub fn get_torrents() -> String {
        String::from("/torrents")
    }
    pub fn get_torrent_info(info_hash: &InfoHash) -> String {
        format!("/torrents/{}/stats/v1", info_hash)
    }
    pub fn add_torrent() -> String {
        String::from("/torrents")
    }
    pub fn forget_torrent(info_hash: &InfoHash) -> String {
        format!("/torrents/{}/forget", info_hash)
    }
    pub fn delete_torrent(info_hash: &InfoHash) -> String {
        format!("/torrents/{}/delete", info_hash)
    }
    pub fn pause_torrent(info_hash: &InfoHash) -> String {
        format!("/torrents/{}/pause", info_hash)
    }
    pub fn start_torrent(info_hash: &InfoHash) -> String {
        format!("/torrents/{}/start", info_hash)
    }
}
