use color_eyre::Result;

use crate::app::App;

mod action;
mod app;
mod connector_worker;
mod connectors;
mod errors;
mod mode;
mod settings;
mod terminal;
mod torrent;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    let mut app = App::new()?;
    app.run().await?;
    Ok(())
}
