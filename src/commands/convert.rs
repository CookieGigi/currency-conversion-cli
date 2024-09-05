use crate::{cli::ConvertArgs, config::Config};

use anyhow::Result;

use currency_conversion::convert::convert_currency::convert;

#[cfg(not(tarpaulin_include))]
pub async fn run_convert(config: &Config, args: &ConvertArgs) -> Result<()> {
    use anyhow::bail;
    use currency_conversion::storage::{
        common::StorageType, psql::PSQLStorageManager, tsv::TSVStorageManager,
    };
    use rust_decimal::Decimal;

    tracing::info!("Convert begin");
    tracing::debug!("{:?}", args);

    let res: Decimal;
    if let StorageType::TSV(settings) = config.conversion_rates_storage.clone() {
        let storage_manager = TSVStorageManager::from_settings(settings)?;
        res = convert(
            &storage_manager,
            &config.base,
            &args.from,
            &args.to,
            args.value,
        )
        .await?;
    } else if let StorageType::PSQL(settings) = config.conversion_rates_storage.clone() {
        let storage_manager = PSQLStorageManager::from_settings(settings).await?;
        res = convert(
            &storage_manager,
            &config.base,
            &args.from,
            &args.to,
            args.value,
        )
        .await?;
    }
    else{
        bail!("No storage manager found !")

    }

    println!("{res}");
    tracing::info!("Convert end");
    tracing::debug!("{res}");
    Ok(())
}
