use std::path::PathBuf;

use directories::ProjectDirs;

use color_eyre::eyre::Result;
use config::{Config, File, FileFormat};
use serde::Deserialize;

use crate::settings::{connectors::Connectors, keybindings::KeyBindings, styles::Styles};

pub mod connectors;
pub mod keybindings;
pub mod styles;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub keybindings: KeyBindings,
    pub connectors: Connectors,
    pub styles: Styles,
    pub notification_timeout_millis: u64,
    pub show_help_auto: bool,
    pub player_cmd: String,
    pub auto_insert_torrent: bool,
}

impl Settings {
    pub fn new(config_str: Option<&str>) -> Result<Self> {
        let user_config_file_path = get_config_dir().join("config.toml");

        let mut config_builder = Config::builder();

        if let Some(config_str) = config_str {
            config_builder =
                config_builder.add_source(File::from_str(config_str, FileFormat::Toml));
        } else {
            const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");
            config_builder = config_builder
                .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
                .add_source(File::from(user_config_file_path).required(false));
        }

        let settings = config_builder.build()?;

        Ok(settings.try_deserialize()?)
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Some(project_dir) = get_project_dir() {
        project_dir.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

fn get_project_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "fucaluca", env!("CARGO_PKG_NAME"))
}
