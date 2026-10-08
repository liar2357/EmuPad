use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "emu-pad")]
#[command(version)]
#[command(about = "virtual mouse by touchscreen for Wayland")]
pub struct Args {
    /// Configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Parser, Debug)]
#[command(
    name = "emu-padctl",
    version,
    about = "Control a running EmuPanel instance."
)]
pub struct Args4Ctl {
    pub command: Vec<String>,
}
