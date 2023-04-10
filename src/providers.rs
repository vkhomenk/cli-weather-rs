mod accu_weather;
mod open_weather;

pub use accu_weather::AccuWeather;
pub use open_weather::OpenWeather;

use crate::cli::ProviderKind;
use crate::config::Config;

use anyhow::Result;
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::time::Duration;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
const TIMEOUT_SECONDS: u64 = 10;

/// Standart for reporting weather data.
pub struct Weather {
    pub place: String,
    pub date: Option<NaiveDate>,
    pub details: String,
}

impl Weather {
    pub fn print(&self) {
        let date = match self.date {
            Some(day) => format!("on {}", day.format("%d.%m")),
            None => "right now".to_string(),
        };

        println!("Weather in {} {}:\n{}", self.place, date, self.details);
    }
}

pub trait WeatherProvider {
    /// Requests forecast info and formats it as [`Weather`].
    ///
    /// # Safety
    ///
    /// Returns `Err` if there was any error while communicating with provider.
    fn get_weather(&self, address: String, date: Option<NaiveDate>) -> Result<Weather>;
}

/// Wrapper that generalises different providers
pub struct ProviderHandle {
    provider: Box<dyn WeatherProvider>,
}

impl ProviderHandle {
    /// Creates a general handle to specific provider depending on configuration or if specified.
    pub fn new(config: Config, requested_provider: Option<ProviderKind>) -> Result<Self> {
        let kind = match requested_provider {
            Some(kind) => kind,
            None => config.default_provider()?.parse()?,
        };
        let api_key = config.get_api_key(&kind.full_name())?.clone();
        let client = Client::builder()
            .user_agent(APP_USER_AGENT)
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .build()?;

        Ok(Self {
            provider: match kind {
                ProviderKind::Open => Box::new(OpenWeather::new(api_key, client)),
                ProviderKind::Accu => Box::new(AccuWeather::new(api_key, client)),
            },
        })
    }

    pub fn get_weather(self, address: String, date: Option<NaiveDate>) -> Result<Weather> {
        self.provider.get_weather(address, date)
    }
}
