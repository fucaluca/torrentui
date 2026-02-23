use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    config_file: Option<String>,
}

impl Args {
    pub fn config_file(self) -> Option<String> {
        self.config_file
    }
}
