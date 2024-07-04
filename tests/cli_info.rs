use std::path::PathBuf;

use assert_cmd::Command;
use currency_conversion::{
    common::{conversion_rate::ConversionRate, supported_symbols::Symbols},
    storage::{
        common::StorageManager,
        tsv::{TSVStorageManager, TSVStorageSettings},
    },
};
use currency_conversion_cli::config::Config;
use rust_decimal_macros::dec;

#[test]
fn cli_info() -> Result<(), Box<dyn std::error::Error>> {
    let conversion_rates = vec![ConversionRate {
        from: "EUR".to_string(),
        to: "USD".to_string(),
        rate: dec!(1.08),
    }];

    let symbols = vec![Symbols {
        code: "EUR".to_string(),
        name: "Euro".to_string(),
    }];

    let dirpath = "./temp/test/cli_info";

    std::fs::create_dir_all(dirpath).unwrap();

    let mut conversion_rates_path = PathBuf::new();

    conversion_rates_path.push(dirpath.to_string() + "/conversion_rate.tsv");
    let tsv_settings_conversion_rates = TSVStorageSettings {
        file_path: conversion_rates_path,
    };
    let storage_manager_conversion_rates =
        TSVStorageManager::from_settings(tsv_settings_conversion_rates.clone());

    StorageManager::update(&storage_manager_conversion_rates, &conversion_rates).unwrap();

    let mut symbols_path = PathBuf::new();

    symbols_path.push(dirpath.to_string() + "/symbols.tsv");
    let tsv_settings_symbols = TSVStorageSettings {
        file_path: symbols_path,
    };
    let storage_manager_symbols = TSVStorageManager::from_settings(tsv_settings_symbols.clone());

    StorageManager::update(&storage_manager_symbols, &symbols).unwrap();

    let config_path = dirpath.to_string() + "/config.toml";
    let config = Config {
        conversion_rates_storage: currency_conversion::storage::common::StorageType::TSV(
            tsv_settings_conversion_rates,
        ),
        base: "EUR".to_string(),
        symbols_storage: currency_conversion::storage::common::StorageType::TSV(
            tsv_settings_symbols,
        ),
        api_key: "test".to_string(),
        ..Default::default()
    };

    confy::store_path(&config_path, config).unwrap();

    let mut cmd = Command::cargo_bin("currency-conversion-cli")?;

    cmd.arg("-vvv")
        .arg("--config-path")
        .arg(config_path)
        .arg("info")
        .arg("--all")
        .assert()
        .success();

    std::fs::remove_dir_all(dirpath).unwrap();

    Ok(())
}
