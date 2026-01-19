use ratatui::{buffer::Buffer, layout::Rect};
use snafu::Snafu;

pub mod torrent_list;
pub use torrent_list::TorrentList;

use crate::settings::Settings;

#[derive(Debug, Snafu)]
pub enum ComponentError {
    #[snafu(display(r#"Failed to draw ui component "{}""#, component))]
    DrawFailed { component: &'static str },
}

pub struct Components<'a> {
    pub torrent_list: TorrentList<'a>,
}

impl<'a> Components<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            torrent_list: TorrentList::new(settings),
        }
    }
}

pub trait Drawable {
    fn draw(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), ComponentError>;
}
