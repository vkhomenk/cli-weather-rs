mod accu_weather;
mod open_weather;
use crate::cli::ProviderKind;
use crate::config::Config;
use accu_weather::AccuWeather;
use anyhow::Result;
use chrono::NaiveDate;
use open_weather::OpenWeather;
use reqwest::blocking::Client;
use std::time::Duration;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
const TIMEOUT_SECONDS: u64 = 10;

#[derive(Default)]
pub struct Weather {
    pub place: String,
    pub date: Option<NaiveDate>,
    pub details: String,
}

impl Weather {
    pub fn print(&self) {
        let date = match self.date {
            None => "right now".to_string(),
            Some(day) => format!("on {}", day.format("%d.%m")),
        };

        println!("Weather in {} {}:\n{}", self.place, date, self.details);
    }
}

pub trait WeatherProvider {
    fn get_weather(&self, address: String, date: Option<NaiveDate>) -> Result<Weather>;
}

pub struct ProviderHandle {
    provider: Box<dyn WeatherProvider>,
}

impl ProviderHandle {
    pub fn new(config: Config, requested_provider: Option<ProviderKind>) -> Result<Self> {
        let kind = match requested_provider {
            Some(kind) => kind,
            None => config.default_provider()?.parse()?,
        };
        let api_key = config.get_key(&kind.to_string())?;
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

    pub fn get_weather(&self, address: String, date: Option<NaiveDate>) -> Result<Weather> {
        self.provider.get_weather(address, date)
    }
}
