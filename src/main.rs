#![warn(
    // clippy::all,
    clippy::restriction,
    // clippy::pedantic,
    // clippy::nursery,
    // clippy::cargo,
)]

mod cli;
mod config;
mod providers;
use cli::{Cli, Command};
use config::Config;
use providers::ProviderHandle;

use clap::Parser;

fn run(cli: Cli, mut config: Config) -> anyhow::Result<()> {
    match cli.command {
        Command::Configure {
            provider,
            api_key,
            set_default,
        } => config.configure_provider(provider, api_key, set_default),
        Command::SetDefault { provider } => config.set_default_provider(provider),
        Command::Get {
            address,
            date,
            provider,
        } => {
            let provider_api = ProviderHandle::new(config, provider)?;
            let weather = provider_api.get_weather(address, date)?;
            weather.print();

            Ok(())
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let config = Config::load().unwrap_or_default();

    if let Err(err) = run(cli, config) {
        eprintln!("{err}");
    }
}
