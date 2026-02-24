use color_eyre::Result;

use crate::app::App;

mod app;
mod args;
mod connector_worker;
mod connectors;
mod domain;
mod errors;
mod logging;
mod settings;
mod terminal;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;
    let mut app = App::new()?;
    app.run().await?;
    Ok(())
}
