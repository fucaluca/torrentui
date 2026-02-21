use ratatui::{buffer::Buffer, layout::Rect};

pub mod add_torrent;
pub mod notifications;
pub mod torrent_list;
pub mod which_key;

pub use notifications::Notifications;
pub use torrent_list::TorrentList;

use crate::{
    settings::Settings,
    ui::{add_torrent::AddTorrent, which_key::WhichKey},
};

pub struct Components {
    pub torrent_list: TorrentList,
    pub notifications: Notifications,
    pub which_key: WhichKey,
    pub add_torrent: AddTorrent,
}

impl Components {
    pub fn new(settings: &Settings) -> Self {
        Self {
            torrent_list: TorrentList::new(),
            notifications: Notifications::new(),
            which_key: WhichKey::new(),
            add_torrent: AddTorrent::new(settings),
        }
    }
}

pub trait Drawable {
    fn draw(&mut self, buf: &mut Buffer, area: Rect, settings: &Settings);
}
