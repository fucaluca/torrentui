use color_eyre::Result;

use crate::{
    app::App,
    settings::{ConfigSource, Settings, get_config_dir},
};

mod actions;
mod app;
mod app_state;
mod errors;
mod key_mode;
mod settings;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;

    let config_file_path = get_config_dir().join("config.toml");
    let config_source = ConfigSource::File(config_file_path);
    let settings = Settings::new(config_source)?;
    let mut app = App::new(&settings)?;
    app.run().await?;
    Ok(())
}
