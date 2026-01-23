use ratatui::{buffer::Buffer, layout::Rect};

pub mod notifications;
pub mod torrent_list;
pub mod which_key;

pub use notifications::Notifications;
pub use torrent_list::TorrentList;

use crate::{settings::Settings, ui::which_key::WhichKey};

pub struct Components<'a> {
    pub torrent_list: TorrentList<'a>,
    pub notifications: Notifications<'a>,
    pub which_key: WhichKey<'a>,
}

impl<'a> Components<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            torrent_list: TorrentList::new(settings),
            notifications: Notifications::new(settings),
            which_key: WhichKey::new(settings),
        }
    }
}

pub trait Drawable {
    fn draw(&mut self, buf: &mut Buffer, area: Rect);
}
