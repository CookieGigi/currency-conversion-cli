use std::path::{Path, PathBuf};

use assert_cmd::Command;
use currency_conversion::storage::tsv::TSVStorageSettings;
use currency_conversion_cli::config::Config;
use httpmock::{Method::GET, MockServer};
use serde_json::json;

#[test]
fn cli_update() -> Result<(), Box<dyn std::error::Error>> {
    // server mocking
    // param
    let api_key = "123";
    let base = "EUR";
    let server_response_conversion_rate = json!({
        "success": true,
        "timestamp": 1519296206,
        "base": "EUR",
        "date": "2021-03-17",
        "rates": {
            "GBP": 0.72007,
            "JPY": 107.346001,
            "USD": 0.813399,
        }
    });
    let server_response_symbols = json!(
    {
      "success": true,
      "symbols": {
        "AED": "United Arab Emirates Dirham",
        "AFN": "Afghan Afghani",
        "ALL": "Albanian Lek",
        "AMD": "Armenian Dram",
        }
    }
    );

    let server = MockServer::start();

    let dirpath = "./temp/test/cli_update";

    std::fs::create_dir_all(dirpath).unwrap();

    // mock latest endpoint
    let mock_conversion_rate = server.mock(|when, then| {
        when.method(GET)
            .path("/latest")
            .query_param("access_key", api_key)
            .query_param("base", base);
        then.status(200)
            .header("content-type", "application/json")
            .json_body(server_response_conversion_rate);
    });

    let mock_symbols = server.mock(|when, then| {
        when.method(GET)
            .path("/symbols")
            .query_param("access_key", api_key);
        then.status(200)
            .header("content-type", "application/json")
            .json_body(server_response_symbols);
    });

    // modify config
    let config_path = dirpath.to_string() + "/config.toml";
    let mut conversion_rate_path =PathBuf::new();
    conversion_rate_path.push( dirpath.to_string() + "/conversion_rate.tsv");
    let conversion_rates_tsv_settings = TSVStorageSettings {
        file_path: conversion_rate_path.clone(),
    };
    let mut symbols_path = PathBuf::new();
    symbols_path.push(dirpath.to_string() + "/symbols.tsv");
    let symbols_tsv_settings = TSVStorageSettings {
        file_path: symbols_path.clone(),
    };
    #[allow(clippy::needless_update)]
    let config = Config {
        latest_endpoint_url: server.url("/latest") + "?access_key={api_key}&base={base}",
        symbols_endpoint_url: server.url("/symbols") + "?access_key={api_key}",
        base: base.to_string(),
        api_key: api_key.to_string(),
        conversion_rates_storage: currency_conversion::storage::common::StorageType::TSV(
            conversion_rates_tsv_settings,
        ),
        symbols_storage: currency_conversion::storage::common::StorageType::TSV(
            symbols_tsv_settings,
        ),
        ..Default::default()
    };

    confy::store_path(&config_path, config).unwrap();

    // exec command
    let mut cmd = Command::cargo_bin("currency-conversion-cli")?;

    // command working
    cmd.arg("-vvv")
        .arg("--config-path")
        .arg(config_path)
        .arg("update")
        .arg("--all")
        .assert()
        .success();

    // server assert
    mock_symbols.assert();
    mock_conversion_rate.assert();

    // file is created
    assert!(Path::new(&symbols_path).exists());
    assert!(Path::new(&conversion_rate_path).exists());

    // check file content
    let mut csv_rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(&conversion_rate_path)?;

    // header
    {
        let headers = csv_rdr.headers()?;
        assert_eq!(headers, vec!["from", "to", "rate"]);
    }

    // content
    let first_row = csv_rdr.records().next();
    assert!(first_row.is_some());
    assert!(first_row.unwrap().is_ok());

    std::fs::remove_dir_all(dirpath).unwrap();

    Ok(())
}
