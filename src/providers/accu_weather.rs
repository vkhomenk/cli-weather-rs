use super::{Weather, WeatherProvider};
use anyhow::{bail, Error, Result};
use chrono::NaiveDate;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

/// `AccuWeather` API.
pub struct AccuWeather {
    api_key: String,
    client: Client,
}

impl AccuWeather {
    pub fn new(api_key: String, client: Client) -> Self {
        Self { api_key, client }
    }

    /// Request location key.
    fn get_place(&self, address: String) -> Result<Place> {
        let location_url = format!(
            "https://dataservice.accuweather.com/locations/v1/cities/autocomplete?q={}&apikey={}",
            address, self.api_key
        );

        let rspns: Value = self.client.get(location_url).send()?.json()?;
        if !rspns["Code"].is_null() {
            bail!("Error {}: {}", rspns["Code"], rspns["Message"]);
        }

        serde_json::from_value::<Vec<Place>>(rspns)
            .into_iter()
            .flatten()
            .next()
            .ok_or(Error::msg(format!("No location data for {address}")))
    }
}

impl WeatherProvider for AccuWeather {
    /// Request location key and then request [`Weather`]
    fn get_weather(&self, address: String, date: Option<NaiveDate>) -> Result<Weather> {
        let place = self.get_place(address)?;

        let weather_url = format!(
            "https://dataservice.accuweather.com/forecasts/v1/daily/5day/{}?details=true&metric=true&apikey={}",
            place.key, self.api_key
        );

        let rspns: Value = self.client.get(weather_url).send()?.json()?;
        if !rspns["Code"].is_null() {
            bail!("Error {}: {}", rspns["Code"], rspns["Message"]);
        }

        let forecast_list: Vec<Forecast> = serde_json::from_value(rspns["DailyForecasts"].clone())
            .map_err(|_| Error::msg("Undefined weather format"))?;

        let possible_forecast = match date {
            None => forecast_list.first(),
            Some(day) => forecast_list
                .iter()
                .find(|forecast| forecast.date.starts_with(&day.to_string())),
        };

        let Some(wthr_that_day) = possible_forecast else {
            bail!("No forecast for this day");
        };

        let short_desc = &wthr_that_day.day.short_phrase;
        let temp = (wthr_that_day.temperature.maximum.value
            + wthr_that_day.temperature.minimum.value)
            / 2.0;
        let temp_feel = (wthr_that_day.real_feel_temperature.maximum.value
            + wthr_that_day.real_feel_temperature.minimum.value)
            / 2.0;
        let wind = wthr_that_day.day.wind.speed.value;

        Ok(Weather {
            date,
            place: format!("{}, {}", place.localized_name, place.country.localized_name),
            details: format!(
                "{short_desc}\nTemperature: {temp:.2}\nFeels like: {temp_feel:.2}\nWind Speed: {wind:.2}"
            ),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Place {
    key: String,
    localized_name: String,
    country: Country,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Country {
    localized_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Forecast {
    date: String,
    temperature: MinMaxData,
    real_feel_temperature: MinMaxData,
    day: Day,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MinMaxData {
    minimum: DataUnit,
    maximum: DataUnit,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DataUnit {
    value: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Day {
    short_phrase: String,
    wind: Wind,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Wind {
    speed: DataUnit,
}
