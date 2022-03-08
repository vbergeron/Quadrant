use std::path::PathBuf;

use clap;
use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(version)]
pub struct Args {
    #[clap(long)]
    pub init_schema: bool,

    #[clap(long, default_value = "INFO")]
    pub log_level: String,

    #[clap(long, short, default_value = "./data")]
    pub datadir: PathBuf,

    #[clap(long, default_value = "https://rpc.atomscan.com")]
    pub rpc: String,

    #[clap(long, default_value_t = 5200791)]
    pub from_block: u32,

    #[clap(long, default_value_t = 0xFFFFFFFF)]
    pub to_block: u32,
}
