use crate::cli::ProviderKind;
use anyhow::{Error, Result};
use dialoguer::{Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use strum::IntoEnumIterator;

const CONFIG_FILE_PATH: &str = "weather-config.json";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    providers: HashMap<String, String>,
    default: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let contents = read_to_string(CONFIG_FILE_PATH)?;
        serde_json::from_str(&contents).map_err(Error::msg)
    }

    pub fn configure_provider(
        &mut self,
        specified_provider: Option<ProviderKind>,
        specified_api_key: Option<String>,
        set_default: bool,
    ) -> Result<()> {
        let provider = if let Some(name) = specified_provider {
            name.to_string()
        } else {
            let supported: Vec<_> = ProviderKind::iter().map(|p| p.full_name()).collect();
            select_provider(supported, "Select provider to configure")?
        };

        let api_key = match specified_api_key {
            Some(key) => key,
            None => Input::new()
                .with_prompt("Provide API key")
                .interact_text()?,
        };

        let no_other_provider = || !self.providers.keys().any(|existing| provider != **existing);
        if set_default || no_other_provider() {
            self.default = Some(provider.clone());
        }

        self.providers.remove(&provider);
        self.providers.insert(provider, api_key);

        self.save()
    }

    pub fn set_default_provider(&mut self, specified_name: Option<String>) -> Result<()> {
        self.default_provider()?;

        let configured_provider = if let Some(provider) = specified_name {
            self.get_api_key(&provider)?;
            provider
        } else {
            let configured: Vec<_> = ProviderKind::iter()
                .map(|p| p.to_string())
                .filter(|p| self.providers.contains_key(p))
                .collect();

            select_provider(configured, "Select provider to use by default")?
        };

        self.default = Some(configured_provider);
        self.save()
    }

    pub fn get_api_key(&self, provider: &str) -> Result<&String> {
        self.providers
            .get(provider)
            .ok_or(Error::msg("This provider is not configured"))
    }

    pub fn default_provider(&self) -> Result<&String> {
        self.default
            .as_ref()
            .ok_or(Error::msg("No providers configured"))
    }

    fn save(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self)?;
        write(CONFIG_FILE_PATH, contents).map_err(Error::msg)
    }
}

/// Prompts to select provider with arrow keys
fn select_provider(options: Vec<String>, prompt: &str) -> Result<String> {
    let index = Select::new()
        .with_prompt(prompt)
        .items(&options)
        .default(0)
        .interact()?;

    Ok(options
        .into_iter()
        .nth(index)
        .expect("Selection out of bounds"))
}
