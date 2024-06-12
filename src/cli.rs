use crate::Error;
use clap::{ArgGroup, Parser};
use log::LevelFilter;
use std::{net::IpAddr, str::FromStr};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
#[clap(group(ArgGroup::new("required")))]
pub struct Opt {
    #[clap(short, long, display_order = 3)]
    /// Enable debug logging level
    pub debug: bool,
    #[clap(short = 'a', long = "mining-address", display_order = 0)]
    /// The Spectre address for the miner reward
    pub mining_address: String,
    #[clap(short = 's', long = "spectred-address", default_value = "127.0.0.1", display_order = 1)]
    /// The IP of the spectred instance
    pub spectred_address: String,

    #[clap(long = "devfund", display_order = 6)]
    /// Mine a percentage of the blocks to the Spectre devfund [default: Off]
    pub devfund_address: Option<String>,

    #[clap(long = "devfund-percent", default_value = "1", display_order = 7, value_parser = parse_devfund_percent)]
    /// The percentage of blocks to send to the devfund
    pub devfund_percent: u16,

    #[clap(short, long, display_order = 2)]
    /// Spectred port [default: Mainnet = 18110, Testnet = 18210]
    port: Option<u16>,

    #[clap(long, display_order = 4)]
    /// Use testnet instead of mainnet [default: false]
    testnet: bool,
    #[clap(short = 't', long = "threads", display_order = 5)]
    /// Amount of miner threads to launch [default: number of logical cpus]
    pub num_threads: Option<u16>,
    #[clap(long = "mine-when-not-synced", display_order = 8)]
    /// Mine even when spectred says it is not synced, only useful when passing `--allow-submit-block-when-not-synced` to spectred  [default: false]
    pub mine_when_not_synced: bool,
    #[clap(long = "throttle", display_order = 9)]
    /// Throttle (milliseconds) between each pow hash generation (used for development testing)
    pub throttle: Option<u64>,
    #[clap(long, display_order = 10)]
    /// Output logs in alternative format (same as spectred)
    pub altlogs: bool,
}

fn parse_devfund_percent(s: &str) -> Result<u16, &'static str> {
    let err = "devfund-percent should be --devfund-percent=XX.YY up to 2 numbers after the dot";
    let mut splited = s.split('.');
    let prefix = splited.next().ok_or(err)?;
    // if there's no postfix then it's 0.
    let postfix = splited.next().ok_or(err).unwrap_or("0");
    // error if there's more than a single dot
    if splited.next().is_some() {
        return Err(err);
    };
    // error if there are more than 2 numbers before or after the dot
    if prefix.len() > 2 || postfix.len() > 2 {
        return Err(err);
    }
    let postfix: u16 = postfix.parse().map_err(|_| err)?;
    let prefix: u16 = prefix.parse().map_err(|_| err)?;
    // can't be more than 99.99%,
    if prefix >= 100 || postfix >= 100 {
        return Err(err);
    }
    Ok(prefix * 100 + postfix)
}

impl Opt {
    pub fn process(&mut self) -> Result<(), Error> {
        if self.spectred_address.is_empty() {
            self.spectred_address = "127.0.0.1".to_string();
        }

        if !self.spectred_address.starts_with("grpc://") {
            IpAddr::from_str(&self.spectred_address)?;
            let port = self.port();
            self.spectred_address = format!("grpc://{}:{}", self.spectred_address, port);
        }
        log::info!("Spectred address: {}", self.spectred_address);

        Ok(())
    }

    fn port(&mut self) -> u16 {
        *self.port.get_or_insert(if self.testnet { 18210 } else { 18110 })
    }

    pub fn log_level(&self) -> LevelFilter {
        if self.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        }
    }
}
