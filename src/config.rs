use crate::cli::ProviderKind;
use anyhow::{Error, Result};
use dialoguer::{Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use strum::IntoEnumIterator;

const CONFIG_FILE_PATH: &str = "weather-config.json";

/// Main configuration struct
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    providers: HashMap<String, String>,
    default: Option<String>,
}

impl Config {
    /// Read configuration from file
    pub fn load() -> Result<Self> {
        let contents = read_to_string(CONFIG_FILE_PATH)?;
        serde_json::from_str(&contents).map_err(Error::msg)
    }

    /// Sets provider, either specified by argument or selected by arrow keys.
    /// Sets its API key, either specified with --api-key/-k flag, or from prompted input.
    /// If it's first configuration, or if --default/-d flag is used, sets the provider as default.
    pub fn configure_provider(
        &mut self,
        specified_provider: Option<ProviderKind>,
        specified_api_key: Option<String>,
        set_default: bool,
    ) -> Result<()> {
        let provider = if let Some(kind) = specified_provider {
            kind.full_name()
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

        self.providers.insert(provider.clone(), api_key);

        let is_first_provider = || !self.providers.keys().any(|existing| provider != **existing);
        if set_default || is_first_provider() {
            self.default = Some(provider);
        }

        self.save()
    }

    /// Set provider to use by default. If None is passed - asks to select by arrow keys
    pub fn set_default_provider(&mut self, specified_provider: Option<ProviderKind>) -> Result<()> {
        self.default_provider()?;

        let configured_provider = if let Some(kind) = specified_provider {
            let provider = kind.full_name();
            self.get_api_key(&provider)?;
            provider
        } else {
            let configured: Vec<_> = ProviderKind::iter()
                .map(|p| p.full_name())
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
        let contents = serde_json::to_string_pretty(self)?;
        write(CONFIG_FILE_PATH, contents).map_err(Error::msg)
    }
}

/// Prompts to select provider with arrow keys
fn select_provider(mut options: Vec<String>, prompt: &str) -> Result<String> {
    let index = Select::new()
        .with_prompt(prompt)
        .items(&options)
        .default(0)
        .interact()
        .map_err(Error::msg)?;

    Ok(options.swap_remove(index))
}
