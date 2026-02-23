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
    #[cfg_attr(test, serde(default))]
    pub keybindings: KeyBindings,
    #[cfg_attr(test, serde(default))]
    pub connectors: Connectors,
    #[cfg_attr(test, serde(default))]
    pub styles: Styles,
    #[cfg_attr(test, serde(default))]
    pub notification_timeout_millis: u64,
    #[cfg_attr(test, serde(default))]
    pub show_help_auto: bool,
    #[cfg_attr(test, serde(default))]
    pub player_cmd: String,
    #[cfg_attr(test, serde(default))]
    pub auto_insert_torrent: bool,
}

impl Settings {
    pub fn new(custom_config_path: Option<&str>) -> Result<Self> {
        let user_config_file_path = custom_config_path
            .map(PathBuf::from)
            .unwrap_or(get_config_dir().join("config.toml"));

        const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");
        let settings = Config::builder()
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_source(File::from(user_config_file_path).required(false))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    #[cfg(test)]
    pub fn test_settings(config_str: impl Into<String>) -> Result<Self> {
        let settings = Config::builder()
            .add_source(File::from_str(&config_str.into(), FileFormat::Toml))
            .build()?;
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
