use color_eyre::Result;

mod errors;
mod tui;

fn main() -> Result<()> {
    crate::errors::init()?;
    Ok(())
}
