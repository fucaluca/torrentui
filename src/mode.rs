use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum KeyMode {
    #[default]
    TorrentList,
    AddTorrent,
}
