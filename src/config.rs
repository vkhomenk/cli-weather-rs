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
        let select_provider = || {
            let supported_providers: Vec<_> = ProviderKind::iter().map(|p| p.full_name()).collect();

            Select::new()
                .with_prompt("Select provider to configure")
                .items(&supported_providers)
                .default(0)
                .interact()
                .ok()
                .and_then(|index| ProviderKind::iter().nth(index))
        };

        let provider = specified_provider
            .or_else(select_provider)
            .ok_or(Error::msg("Unknown provider"))?
            .to_string();

        let api_key = match specified_api_key {
            Some(key) => key,
            None => Input::new()
                .with_prompt("Provide API key")
                .interact_text()?,
        };

        self.providers.remove(&provider);
        self.providers.insert(provider.clone(), api_key);

        let no_other_provider = !self.providers.keys().any(|existing| provider != **existing);

        if set_default || no_other_provider {
            self.default = Some(provider);
        }

        self.save()
    }

    pub fn set_default_provider(&mut self, specified_name: Option<String>) -> Result<()> {
        self.default_provider()?;

        let configured_provider = if let Some(provider) = specified_name {
            self.get_key(&provider)?;
            provider
        } else {
            let configured_providers: Vec<String> = ProviderKind::iter()
                .map(|p| p.to_string())
                .filter(|provider| self.providers.contains_key(provider))
                .collect();
            let selected = Select::new()
                .with_prompt("Select provider to use by default")
                .items(&configured_providers)
                .default(0)
                .interact()?;

            configured_providers[selected].clone()
        };

        self.default = Some(configured_provider);
        self.save()
    }

    pub fn get_key(&self, provider: &str) -> Result<String> {
        self.providers
            .get(provider)
            .cloned()
            .ok_or(Error::msg("This provider is not configured"))
    }

    pub fn default_provider(&self) -> Result<String> {
        self.default
            .clone()
            .ok_or(Error::msg("No providers configured"))
    }

    fn save(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self)?;
        write(CONFIG_FILE_PATH, contents).map_err(Error::msg)
    }
}
