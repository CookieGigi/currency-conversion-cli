use std::path::PathBuf;

use assert_cmd::Command;
use currency_conversion::{
    common::supported_symbols::Symbols,
    storage::{
        common::StorageManager,
        tsv::{TSVStorageManager, TSVStorageSettings},
    },
};
use currency_conversion_cli::config::Config;
use predicates::prelude::predicate;

#[tokio::test]
async fn cli_get_symbols() -> Result<(), Box<dyn std::error::Error>> {
    let data = vec![
        Symbols {
            code: "EUR".to_string(),
            name: "Euro".to_string(),
        },
        Symbols {
            code: "USD".to_string(),
            name: "United state dollard".to_string(),
        },
    ];

    let dirpath = "./temp/test/cli_get_symbols";

    std::fs::create_dir_all(dirpath).unwrap();

    let mut path = PathBuf::new();
    path.push(dirpath.to_string() + "/symbols.tsv");

    let tsv_settings = TSVStorageSettings { file_path: path };
    let storage_manager = TSVStorageManager::from_settings(tsv_settings.clone()).unwrap();

    StorageManager::update(&storage_manager, &data).await.unwrap();

    let config_path = dirpath.to_string() + "/config.toml";
    let config = Config {
        symbols_storage: currency_conversion::storage::common::StorageType::TSV(tsv_settings),
        api_key: "test".to_string(),
        ..Default::default()
    };

    confy::store_path(&config_path, config).unwrap();

    let mut cmd = Command::cargo_bin("currency-conversion-cli")?;

    cmd.arg("-vvv")
        .arg("--config-path")
        .arg(config_path)
        .arg("list")
        .arg("symbols")
        .assert()
        .success()
        .stdout(predicate::str::contains("EUR : Euro"));

    std::fs::remove_dir_all(dirpath).unwrap();

    Ok(())
}
