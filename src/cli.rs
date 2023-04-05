use anyhow::{Error, Result};
use chrono::{Datelike, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use std::{fmt::Display, str::FromStr};
use strum::EnumIter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Set up provider and its API key
    Configure {
        #[command(subcommand)]
        provider: Option<ProviderKind>,

        /// Specify API key
        #[arg(long, short = 'k')]
        api_key: Option<String>,

        /// Use this provider by default
        #[arg(long, short = 'd')]
        set_default: bool,
    },

    /// Set default provider
    SetDefault { provider: Option<String> },

    /// Get weather at specified address
    Get {
        address: String,

        /// Specify date (up to 5 days ahead)
        #[arg(long, short = 'd', value_parser = parse_date)]
        date: Option<NaiveDate>,

        /// Specify provider
        #[arg(long, short = 'p')]
        provider: Option<ProviderKind>,
    },
}

#[derive(Subcommand, Clone, EnumIter)]
pub enum ProviderKind {
    /// OpenWeather
    Open,
    /// AccuWeather
    Accu,
}

impl ProviderKind {
    pub fn full_name(&self) -> String {
        let s = match self {
            Self::Open => "OpenWeather",
            Self::Accu => "AccuWeather",
        };
        s.into()
    }
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Open => "open",
            Self::Accu => "accu",
        };
        write!(f, "{s}")
    }
}

impl FromStr for ProviderKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "accu" => Ok(Self::Accu),
            _ => Err(Error::msg("Unknown provider")),
        }
    }
}

fn parse_date(date: &str) -> anyhow::Result<NaiveDate> {
    let current_date = Utc::now().naive_utc();
    let full_date = format!("{}.{}", date, current_date.year());

    NaiveDate::parse_from_str(&full_date, "%d.%m.%Y")
        .map_err(|_| Error::msg("Accepted date format: day.month"))
}
