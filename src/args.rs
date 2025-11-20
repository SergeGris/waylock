use clap::Parser;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(name = "Waylock", version, about = "Wayland session lock")]
pub struct Args {
    #[command(flatten)]
    pub config: Config,

    #[arg(short = 'd', long)]
    pub daemonize: bool,
}
