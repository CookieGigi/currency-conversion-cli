use std::collections::HashMap;

use anyhow::Result;

use crate::{cli::InfoArgs, config::Config};

use self::{common::Info, info_config::get_config_info};

mod common;
mod info_config;

#[cfg(not(tarpaulin_include))]
pub async fn run_info(config: Config, args: &InfoArgs, config_path: Option<String>) -> Result<()> {
    use currency_conversion::storage::common::{
        get_conversion_rate_storage_manager, get_symbols_storage_manager, StorageManager,
    };

    let mut infos: HashMap<&str, Info> = HashMap::new();

    tracing::info!("Info begin");
    tracing::debug!("{:?}", args);

    // Symbols
    if args.symbols || args.all {
        let storage_manager = get_symbols_storage_manager(config.symbols_storage.clone());
        infos.insert(
            "symbols",
            Info::Symbols(StorageManager::get_data_info(&storage_manager).await?),
        );
    }

    // Conversion rate

    if args.conversion_rates || args.all {
        let storage_manager =
            get_conversion_rate_storage_manager(config.conversion_rates_storage.clone());
        infos.insert(
            "conversion_rates",
            Info::ConversionRates(StorageManager::get_data_info(&storage_manager).await?),
        );
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
