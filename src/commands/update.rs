use crate::config::Config;

use crate::cli::UpdateArgs;
use anyhow::Result;
use currency_conversion::{storage::common::StorageType, update::{
    update_converison_rates::update_conversion_rates, update_symbols::update_symbols,
}};
use currency_conversion::storage::common::{
        get_conversion_rate_storage_manager, get_symbols_storage_manager,
    };



#[cfg(not(tarpaulin_include))]
pub async fn run_update(config: &Config, args: &UpdateArgs) -> Result<()> {
        tracing::debug!("Update arguments : {:?}", args);

    let handle_symbols = run_update_symbols(args.all || args.symbols, config.symbols_storage.clone(), config.symbols_endpoint_url.clone(), config.api_key.clone());

    let handle_conversion_rates = run_update_conversion_rates(args.all || args.conversion_rates, config.conversion_rates_storage.clone(), config.latest_endpoint_url.clone(), config.api_key.clone(), config.base.clone());

    handle_symbols.await?;
    handle_conversion_rates.await?;
    Ok(())
}

async fn run_update_symbols(update_flag : bool, storage_settings : StorageType, endpoint_url : String, api_key: String) -> Result<()>{
   if update_flag {
        tracing::info!("Update symbols begin");

        let storage_manager = get_symbols_storage_manager(storage_settings);

        update_symbols(
            &endpoint_url,
            &api_key,
            &storage_manager,
        ).await?;

        tracing::info!("Update symbols end");
    }

   Ok(())
}

async fn run_update_conversion_rates(update_flag : bool, storage_settings : StorageType, endpoint_url : String, api_key: String, base : String) -> Result<()>{
   if update_flag {
        tracing::info!("Update conversion rates begin");

        let storage_manager = get_conversion_rate_storage_manager(storage_settings);

        update_conversion_rates(
            &endpoint_url,
            &api_key,
            &base,
            &storage_manager,
        ).await?;

        tracing::info!("Update conversion rates end");
    }

   Ok(())
}
