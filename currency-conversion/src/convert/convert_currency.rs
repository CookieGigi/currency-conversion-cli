use rust_decimal::Decimal;

use crate::{common::conversion_rate::ConversionRate, storage::common::StorageManager};

use anyhow::Result;

/// Convert a `value` `from` a currency `to` another
pub fn convert<T>(
    conversion_rates_storage_manager: &T,
    base: &str,
    from: &str,
    to: &str,
    value: Decimal,
) -> Result<Decimal>
where
    T: StorageManager<ConversionRate>,
{
    let conversion_rates = conversion_rates_storage_manager.get_all()?;

    let rate = ConversionRate::get_conversion_rate(base, &conversion_rates, from, to)?;

    Ok(value * rate.rate)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use rust_decimal_macros::dec;

    use crate::{
        common::conversion_rate::ConversionRate,
        storage::{common::StorageManager, tsv::TSVStorageManager},
    };

    fn setup(dirpath: String, data: Vec<ConversionRate>) -> TSVStorageManager {
        std::fs::create_dir_all(&dirpath).unwrap();

        let mut path = PathBuf::new();

        path.push(dirpath);
        path.push("conversion_rate.tsv");

        let storage_manager = TSVStorageManager::build(path);

        storage_manager.update(&data).unwrap();

        storage_manager
    }

    #[test]
    fn convert() {
        let dirpath = "./temp/test/convert/convert_currency/";
        let from = "EUR".to_string();
        let to = "USD".to_string();
        let data = vec![ConversionRate {
            from: from.clone(),
            to: to.clone(),
            rate: dec!(1.08),
        }];
        let storage_manager = setup(dirpath.to_string(), data);

        let res = super::convert(&storage_manager, "EUR", &from, &to, dec!(10.0));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), dec!(10.8));
    }
}
