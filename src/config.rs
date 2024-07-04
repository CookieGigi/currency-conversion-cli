use std::{io::Stdin, path::PathBuf, str::FromStr};

use anyhow::{bail, Result};
use currency_conversion::storage::{common::StorageType, tsv::TSVStorageSettings};
use serde::{Deserialize, Serialize};

/// Config file structure
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Config {
    /// API token
    pub api_key: String,
    /// base currency
    pub base: String,
    /// endpoint url to get supported symbols (param : {api_key})
    pub symbols_endpoint_url: String,
    /// endpoint url to get conversion rates (param : {api_key}, {base})
    pub latest_endpoint_url: String,
    /// Define storage strategy for symbols
    pub symbols_storage: StorageType,
    /// Define storage strategy for conversion_rates
    pub conversion_rates_storage: StorageType,
}

#[cfg(not(tarpaulin_include))]
impl Default for Config {
    fn default() -> Self {
        // If error to find home_dir => panic
        let homedir = home::home_dir().unwrap();

        let mut symbols_file_path = PathBuf::new();
        symbols_file_path.push(&homedir);
        symbols_file_path.push(".currency-conversion-cli/symbols.tsv");

        let mut conversion_rates_file_path = PathBuf::new();
        conversion_rates_file_path.push(&homedir);
        conversion_rates_file_path.push(".currency-conversion-cli/conversion_rates.tsv");

        Config {
            api_key: "#INSERT_API_KEY_HERE#".to_string(),
            base: "EUR".to_string(),
            symbols_storage: StorageType::TSV(TSVStorageSettings {
                file_path: symbols_file_path,
            }),
            conversion_rates_storage: StorageType::TSV(TSVStorageSettings {
                file_path: conversion_rates_file_path,
            }),
            latest_endpoint_url:
                "http://api.exchangeratesapi.io/v1/latest?access_key={api_key}&base={base}"
                    .to_string(),
            symbols_endpoint_url: "http://api.exchangeratesapi.io/v1/symbols?access_key={api_key}"
                .to_string(),
        }
    }
}
#[cfg(not(tarpaulin_include))]
impl Config {
    pub fn prompt_config(&self) -> Result<Config> {
        let mut res = Config::default();
        let stdin = std::io::stdin();
        let mut buffer = String::new();
        println!("Initialization of config file");

        // api key
        println!(
            "api key (required exchange rates api key)(current : {}) : ",
            self.api_key
        );
        stdin.read_line(&mut buffer)?;
        if !buffer.trim().is_empty() {
            res.api_key.clone_from(&buffer.trim().to_string());
        } else if self.api_key != "#INSERT_API_KEY_HERE#" {
            res.api_key.clone_from(&self.api_key);
        } else {
            bail!("API key must be provided !")
        }

        // base
        res.base
            .clone_from(&prompt_string(&stdin, "base currency", &self.base)?);
        // symbols storage strategy
        res.symbols_storage.clone_from(&prompt_storage_strategy(
            &stdin,
            "Symbols storage",
            &self.symbols_storage,
        )?);
        // symbols endpoint
        res.symbols_endpoint_url.clone_from(&prompt_string(
            &stdin,
            "currency symbols endpoint URL",
            &self.symbols_endpoint_url,
        )?);
        // converison rates storage strategy
        res.conversion_rates_storage
            .clone_from(&prompt_storage_strategy(
                &stdin,
                "conversion rates storage",
                &self.conversion_rates_storage,
            )?);
        res.latest_endpoint_url.clone_from(&prompt_string(
            &stdin,
            "conversion rates endpoint URL",
            &self.latest_endpoint_url,
        )?);

        Ok(res)
    }
}

fn prompt_storage_strategy(
    stdin: &Stdin,
    text: &str,
    current_value: &StorageType,
) -> Result<StorageType> {
    println!("{text} (current : {:?} : ", current_value);
    println!("Type :");
    let storage_type = prompt_string_without_text_and_default(stdin)?;

    Ok(match storage_type {
        Some(t) => prompt_storage_type_settings(stdin, t, current_value)?,
        None => current_value.clone(),
    })
}

fn prompt_storage_type_settings(
    stdin: &Stdin,
    storage_type: String,
    current_value: &StorageType,
) -> Result<StorageType> {
    if storage_type.to_lowercase().contains("tsv") {
        let settings = prompt_tsv_settings(stdin)?;
        match settings {
            Some(s) => Ok(StorageType::TSV(s)),
            None => Ok(current_value.clone()),
        }
    } else {
        tracing::error!("\"{storage_type}\" is not recognized as valid storage type. We keep the old configuration.");
        Ok(current_value.clone())
    }
}

fn prompt_tsv_settings(stdin: &Stdin) -> Result<Option<TSVStorageSettings>> {
    println!("File path :");
    let filepath = prompt_string_without_text_and_default(stdin)?;
    match filepath {
        Some(p) => Ok(Some(TSVStorageSettings {
            file_path: PathBuf::from_str(&p)?,
        })),
        None => Ok(None),
    }
}

#[cfg(not(tarpaulin_include))]
fn prompt_string(stdin: &Stdin, text: &str, current_value: &String) -> Result<String> {
    println!("{text} (current : {current_value}) : ");
    let mut buffer = String::new();
    stdin.read_line(&mut buffer)?;
    if buffer.trim().is_empty() {
        return Ok(current_value.clone());
    }

    Ok(buffer.trim().to_string())
}

fn prompt_string_without_text_and_default(stdin: &Stdin) -> Result<Option<String>> {
    let mut buffer = String::new();
    stdin.read_line(&mut buffer)?;
    if buffer.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(buffer.trim().to_string()))
}
