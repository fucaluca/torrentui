pub mod info_hash;
pub mod source;

pub use info_hash::InfoHash;
use serde::Deserialize;
pub use source::Magnet;
pub use source::Source;

#[derive(Debug)]
#[cfg_attr(test, derive(fake::Dummy, Clone))]
pub struct TorrentInfo {
    #[cfg_attr(test, dummy(faker = "Sentence(1..3)"))]
    pub name: String,
    pub info_hash: InfoHash,
    #[cfg_attr(test, dummy(faker = "DirPath()"))]
    pub output_folder: String,
    pub finished: bool,
    pub state: State,
    #[cfg_attr(test, dummy(faker = "0..=10000"))]
    pub downloaded_bytes: usize,
    #[cfg_attr(test, dummy(faker = "0..1000000000"))]
    pub uploaded_bytes: usize,
    #[cfg_attr(test, dummy(faker = "10000..1000000000"))]
    pub total_bytes: usize,
    #[cfg_attr(test, dummy(faker = "0.0..10.0"))]
    pub download_speed_mpbs: f32,
    #[cfg_attr(test, dummy(faker = "0.0..10.0"))]
    pub upload_speed_mpbs: f32,
    #[cfg_attr(test, dummy(faker = "0..1000000"))]
    pub time_remaining_secs: Option<usize>,
    #[cfg_attr(test, dummy(faker = "0..=100"))]
    pub peer_live: u32,
    #[cfg_attr(test, dummy(faker = "100..1000"))]
    pub peer_seen: u32,
}

#[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
#[cfg_attr(test, derive(fake::Dummy, Clone))]
pub enum State {
    Active,
    Paused,
    Initializing,
    Error,
}

#[cfg(test)]
use fake::faker::filesystem::en::DirPath;
#[cfg(test)]
use fake::faker::lorem::en::Sentence;
