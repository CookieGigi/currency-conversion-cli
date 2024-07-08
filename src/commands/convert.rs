use crate::{cli::ConvertArgs, config::Config};

use anyhow::Result;

use currency_conversion::convert::convert_currency::convert;

#[cfg(not(tarpaulin_include))]
pub async fn run_convert(config: &Config, args: &ConvertArgs) -> Result<()> {
    use currency_conversion::storage::common::get_conversion_rate_storage_manager;

    tracing::info!("Convert begin");
    tracing::debug!("{:?}", args);

    let storage_manager =
        get_conversion_rate_storage_manager(config.conversion_rates_storage.clone());

    let res = convert(
        &storage_manager,
        &config.base,
        &args.from,
        &args.to,
        args.value,
    ).await?;

    println!("{res}");
    tracing::info!("Convert end");
    tracing::debug!("{res}");
    Ok(())
}
