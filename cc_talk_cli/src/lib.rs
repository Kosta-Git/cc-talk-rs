use clap::{Parser, Subcommand};

use crate::hopper::HopperCommands;

pub mod coinselector;
pub mod hopper;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Unix domain socket path to connect to the ccTalk bus
    #[arg(short, long, default_value = "/tmp/cctalk.sock")]
    pub sock: String,

    /// Transport timeout in milliseconds
    #[arg(short, long, default_value_t = 100)]
    pub timeout: u64,

    /// Disables echo support on transport layer
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub no_echo: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Hopper {
        address: u8,

        #[command(subcommand)]
        action: HopperCommands,
    },

    Selector {
        address: u8,

        #[command(subcommand)]
        action: coinselector::CoinSelectorCommands,
    },
}
