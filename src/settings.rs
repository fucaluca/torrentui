use std::path::PathBuf;

use directories::ProjectDirs;

use color_eyre::eyre::Result;
use config::{Config, File};
use serde::Deserialize;
mod defaults;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "defaults::default_update_interval")]
    update_torrent_list_interval: u8,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let config_file_path = get_config_dir().join("config.toml");
        let settings = Config::builder()
            .add_source(File::from(config_file_path))
            .build()?;
        Ok(settings.try_deserialize()?)
    }
}

fn get_config_dir() -> PathBuf {
    if let Some(project_dir) = get_project_dir() {
        project_dir.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

fn get_project_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "fucaluca", env!("CARGO_PKG_NAME"))
}
