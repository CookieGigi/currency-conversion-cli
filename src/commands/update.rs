use crate::config::Config;

use crate::cli::UpdateArgs;
use anyhow::Result;
use currency_conversion::{
    common::supported_symbols::Symbols,
    storage::{common::StorageType, psql::PSQLStorageManager, tsv::TSVStorageManager},
    update::{update_converison_rates::update_conversion_rates, update_symbols::update_symbols},
};

#[cfg(not(tarpaulin_include))]
pub async fn run_update(config: &Config, args: &UpdateArgs) -> Result<()> {
    tracing::debug!("Update arguments : {:?}", args);

    let handle_symbols = run_update_symbols(
        args.all || args.symbols,
        config.symbols_storage.clone(),
        config.symbols_endpoint_url.clone(),
        config.api_key.clone(),
    );

    let handle_conversion_rates = run_update_conversion_rates(
        args.all || args.conversion_rates,
        config.conversion_rates_storage.clone(),
        config.latest_endpoint_url.clone(),
        config.api_key.clone(),
        config.base.clone(),
    );

    handle_symbols.await?;
    handle_conversion_rates.await?;
    Ok(())
}

async fn run_update_symbols(
    update_flag: bool,
    storage_settings: StorageType,
    endpoint_url: String,
    api_key: String,
) -> Result<()> {
    if update_flag {
        tracing::info!("Update symbols begin");

        if let StorageType::TSV(settings) = storage_settings {
            let storage_manager = TSVStorageManager::from_settings(settings.clone())?;
            update_symbols(&endpoint_url, &api_key, &storage_manager).await?;
        } else if let StorageType::PSQL(settings) = storage_settings {
            let storage_manager = PSQLStorageManager::from_settings(settings.clone())?;
            update_symbols(&endpoint_url, &api_key, &storage_manager).await?;
        }
        tracing::info!("Update symbols end");
    }

    Ok(())
}

async fn run_update_conversion_rates(
    update_flag: bool,
    storage_settings: StorageType,
    endpoint_url: String,
    api_key: String,
    base: String,
) -> Result<()> {
    if update_flag {
        tracing::info!("Update conversion rates begin");

        if let StorageType::TSV(settings) = storage_settings {
            let storage_manager = TSVStorageManager::from_settings(settings.clone())?;
            update_conversion_rates(&endpoint_url, &api_key, &base, &storage_manager).await?;
        } else if let StorageType::PSQL(settings) = storage_settings {
            let storage_manager = PSQLStorageManager::from_settings(settings.clone())?;
            update_conversion_rates(&endpoint_url, &api_key, &base, &storage_manager).await?;
        }

        tracing::info!("Update conversion rates end");
    }

    Ok(())
}
