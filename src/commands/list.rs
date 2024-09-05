use anyhow::Result;
use currency_conversion::common::conversion_rate::ConversionRate;
use currency_conversion::common::supported_symbols::Symbols;
use currency_conversion::list::list_data::list_data;
use currency_conversion::list::list_data::ListDataItem;
use currency_conversion::storage::common::StorageManager;
use serde::Deserialize;
use serde::Serialize;

use crate::{
    cli::{ListArgs, ListDataSet},
    config::Config,
};

#[cfg(not(tarpaulin_include))]
pub async fn run_list(config: &Config, args: &ListArgs) -> Result<()> {
    use currency_conversion::storage::{
        common::StorageType, psql::PSQLStorageManager, tsv::TSVStorageManager,
    };

    match args.dataset {
        ListDataSet::Symbols => {
            if let StorageType::TSV(settings) = &config.symbols_storage {
                let storage_manager = TSVStorageManager::from_settings(settings.clone())?;
                load_and_list_data::<Symbols>(storage_manager).await?;
            } else if let StorageType::PSQL(settings) = &config.symbols_storage {
                let storage_manager = PSQLStorageManager::from_settings(settings.clone()).await?;
                load_and_list_data::<Symbols>(storage_manager).await?;
            }
        }
        ListDataSet::ConversionRates => {
            if let StorageType::TSV(settings) = &config.symbols_storage {
                let storage_manager = TSVStorageManager::from_settings(settings.clone())?;
                load_and_list_data::<ConversionRate>(storage_manager).await?;
            } else if let StorageType::PSQL(settings) = &config.symbols_storage {
                let storage_manager = PSQLStorageManager::from_settings(settings.clone()).await?;
                load_and_list_data::<ConversionRate>(storage_manager).await?;
            }
        }
    };
    Ok(())
}

#[cfg(not(tarpaulin_include))]
async fn load_and_list_data<T>(storage_manager: impl StorageManager<T>) -> Result<()>
where
    T: ListDataItem + for<'de> Deserialize<'de> + Ord + Serialize,
{
    let mut data: Vec<T> = storage_manager.get_all().await?;

    data.sort();

    println!("{}", list_data(&data)?);

    Ok(())
}
