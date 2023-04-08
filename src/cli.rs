use anyhow::{Error, Result};
use chrono::{Datelike, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use color_print::cformat;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

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
        #[arg(value_parser = parse_provider_kind)]
        provider: Option<ProviderKind>,

        /// Specify API key
        #[arg(long, short = 'k')]
        api_key: Option<String>,

        /// Use this provider by default
        #[arg(long, short = 'd')]
        default: bool,
    },

    /// Set default provider
    SetDefault {
        #[arg(value_parser = parse_provider_kind)]
        provider: Option<ProviderKind>,
    },

    /// Get weather at specified address
    Get {
        address: String,

        /// Specify date (up to 5 days ahead)
        #[arg(value_parser = parse_date)]
        date: Option<NaiveDate>,

        /// Specify provider
        #[arg(long, short = 'p', value_parser = parse_provider_kind)]
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

impl ToString for ProviderKind {
    fn to_string(&self) -> String {
        let s = match self {
            Self::Open => "open",
            Self::Accu => "accu",
        };
        s.into()
    }
}

impl FromStr for ProviderKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "OpenWeather" => Ok(Self::Open),
            "AccuWeather" => Ok(Self::Accu),
            _ => Err(Error::msg("Unknown provider")),
        }
    }
}

/// Custom parser for [`ProviderKind`].
fn parse_provider_kind(input: &str) -> Result<ProviderKind> {
    ProviderKind::iter()
        .find(|kind| input == kind.to_string())
        .ok_or_else(|| {
            let options = ProviderKind::iter()
                .map(|kind| cformat!("'<g>{}</>'", kind.to_string()))
                .collect::<Vec<String>>()
                .join("|");
            Error::msg(format!("\ntry: {}", options))
        })
}

/// Custom parser for date format.
fn parse_date(date: &str) -> Result<NaiveDate> {
    let current_year = Utc::now().naive_utc().year();
    let full_date = format!("{}.{}", date, current_year);

    NaiveDate::parse_from_str(&full_date, "%d.%m.%Y")
        .map_err(|_| Error::msg("Accepted date format: day.month"))
}
