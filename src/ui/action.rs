use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

use crate::torrent::TorrentInfo;

#[derive(Clone)]
pub enum UiAction {
    UpdateTorrentLinst,
}
