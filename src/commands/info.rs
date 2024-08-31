use std::collections::HashMap;

use anyhow::Result;

use crate::{cli::InfoArgs, config::Config};

use self::{common::Info, info_config::get_config_info};

mod common;
mod info_config;

#[cfg(not(tarpaulin_include))]
pub async fn run_info(config: Config, args: &InfoArgs, config_path: Option<String>) -> Result<()> {
    use currency_conversion::{
        common::{conversion_rate::ConversionRate, supported_symbols::Symbols},
        storage::{
            common::{StorageManager, StorageType},
            psql::PSQLStorageManager,
            tsv::TSVStorageManager,
        },
    };

    let mut infos: HashMap<&str, Info> = HashMap::new();

    tracing::info!("Info begin");
    tracing::debug!("{:?}", args);

    // Symbols
    if args.symbols || args.all {
        if let StorageType::TSV(settings) = config.symbols_storage.clone() {
            let storage_manager = TSVStorageManager::from_settings(settings)?;
            infos.insert(
                "symbols",
                Info::Symbols(StorageManager::<Symbols>::get_data_info(&storage_manager).await?),
            );
        } else if let StorageType::PSQL(settings) = config.symbols_storage.clone() {
            let storage_manager = PSQLStorageManager::from_settings(settings)?;
            infos.insert(
                "symbols",
                Info::Symbols(StorageManager::<Symbols>::get_data_info(&storage_manager).await?),
            );
        }
    }

    // Conversion rate

    if args.conversion_rates || args.all {
        if let StorageType::TSV(settings) = config.symbols_storage.clone() {
            let storage_manager = TSVStorageManager::from_settings(settings)?;
            infos.insert(
                "conversion_rates",
                Info::ConversionRates(
                    StorageManager::<ConversionRate>::get_data_info(&storage_manager).await?,
                ),
            );
        } else if let StorageType::PSQL(settings) = config.symbols_storage.clone() {
            let storage_manager = PSQLStorageManager::from_settings(settings)?;
            infos.insert(
                "conversion_rates",
                Info::ConversionRates(
                    StorageManager::<ConversionRate>::get_data_info(&storage_manager).await?,
                ),
            );
        }
    }

    // Config
    if args.config || args.all {
        infos.insert(
            "config",
            Info::Config(get_config_info(config, config_path)?),
        );
    }

    println!("{:?}", infos);
    tracing::info!("Info end");
    tracing::debug!("{:?}", infos);

    Ok(())
}
