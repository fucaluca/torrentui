#![cfg_attr(not(test), allow(dead_code))] // TODO: remove this

pub mod rqbit;

use std::error::Error;

use async_trait::async_trait;

use crate::torrent::{self, InfoHash};

#[async_trait]
pub trait Connector {
    type Error: Error + Send + Sync + 'static;

    async fn get_torrent_list(&self) -> Result<Vec<torrent::TorrentInfo>, Self::Error>;
    async fn add_torrent(&self, torrent_source: torrent::Source) -> Result<(), Self::Error>;
    async fn forget_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), Self::Error>;
    async fn delete_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), Self::Error>;
    async fn pause_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), Self::Error>;
    async fn start_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), Self::Error>;
}
