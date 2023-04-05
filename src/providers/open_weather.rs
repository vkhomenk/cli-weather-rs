use super::{Weather, WeatherProvider};
use anyhow::{bail, Error, Result};
use chrono::NaiveDate;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

pub struct OpenWeather {
    api_key: String,
    client: Client,
}

impl OpenWeather {
    pub fn new(api_key: String, client: Client) -> Self {
        Self { api_key, client }
    }
}

impl WeatherProvider for OpenWeather {
    fn get_weather(&self, address: String, date: Option<NaiveDate>) -> Result<Weather> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?units=metric&q={}&appid={}",
            address, self.api_key
        );

        let rspns: Value = self.client.get(url).send()?.json()?;

        if rspns["cod"] != "200" {
            bail!(format!("Error {}: {}", rspns["cod"], rspns["message"]))
        }

        let rspns: ResponseData =
            serde_json::from_value(rspns).map_err(|_| Error::msg("Undefined weather format"))?;

        let forecast_list = rspns.list;

        let wthr_that_day: Vec<Forecast> = match &date {
            None => forecast_list.into_iter().take(1).collect(),
            Some(day) => forecast_list
                .into_iter()
                .filter(|forecast| forecast.dt_txt.starts_with(&day.to_string()))
                .collect(),
        };

        if wthr_that_day.is_empty() {
            bail!("No forecast for this day");
        }

        let mut country_full_name = rspns.city.country.as_str();
        for country in iso_country::data::all() {
            if rspns.city.country == country.alpha2 {
                country_full_name = country.name;
                break;
            }
        }

        let temp = avrg_by_key(&wthr_that_day, |w| {
            (w.main.temp_min + w.main.temp_max) / 2.0
        });
        let temp_feel = avrg_by_key(&wthr_that_day, |w| w.main.feels_like);
        let hmdt = avrg_by_key(&wthr_that_day, |w| w.main.humidity);
        let wind = avrg_by_key(&wthr_that_day, |w| w.wind.speed);

        Ok(Weather {
            date,
            place: format!("{}, {}", rspns.city.name, country_full_name),
            details: format!(
                "Temperature: {temp:.2}\nFeels like: {temp_feel:.2}\nHumidity: {hmdt}\nWind Speed: {wind:.2}"
            ),
        })
    }
}

#[derive(Deserialize)]
struct ResponseData {
    city: City,
    list: Vec<Forecast>,
}

#[derive(Deserialize)]
struct City {
    name: String,
    country: String,
}

#[derive(Deserialize)]
struct Forecast {
    dt_txt: String,
    main: Main,
    wind: Wind,
}

#[derive(Deserialize)]
struct Main {
    temp_min: f32,
    temp_max: f32,
    feels_like: f32,
    humidity: u32,
}

#[derive(Deserialize)]
struct Wind {
    speed: f32,
}

fn avrg_by_key<F, N>(list: &[Forecast], key: F) -> N
where
    N: std::iter::Sum<N> + std::ops::Div<Output = N> + From<u16>,
    F: FnMut(&Forecast) -> N,
{
    let sum: N = list.iter().map(key).sum();
    let len: N = (list.len() as u16).into();

    sum / len
}
