use std::{collections::HashMap, path::Path};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use anyhow::{bail, Result};

use crate::config::Config;

use super::load_data;

/// Conversion Rates from a currency to another
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ConversionRate {
    pub from: String,
    pub to: String,
    pub rate: Decimal,
}

impl ConversionRate {
    pub fn get_conversion_rate(config: &Config, from: &str, to: &str) -> Result<ConversionRate> {
        let conversion_rates: Vec<ConversionRate> =
            load_data(Path::new(&config.conversion_rates_file_path))?;

        println!("{:?}", conversion_rates);

        let res: ConversionRate;
        if to == config.base {
            let mut search_iter = conversion_rates.iter().filter(|rate| rate.to == from);
            let search_result = search_iter.next();

            if search_result.is_none() {
                bail!("{from} symbols not found !");
            }
            res = ConversionRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: Decimal::new(1, 0) / search_result.unwrap().rate,
            };
        } else if from == config.base {
            let mut search_iter = conversion_rates.iter().filter(|rate| rate.to == to);

            let search_result = search_iter.next();

            if search_result.is_none() {
                bail!("{to} symbols not found !");
            }
            res = search_result.unwrap().clone();
        } else {
            let rate_from = ConversionRate::get_conversion_rate(config, &config.base, from)?;
            let rate_to = ConversionRate::get_conversion_rate(config, &config.base, to)?;

            res = ConversionRate {
                from: from.to_string(),
                to: to.to_string(),
                rate: rate_to.rate / rate_from.rate,
            };
        }

        Ok(res)
    }
}

/// Convert a hashmap (key : to currency, value : rates) and base (from currency) to vec of ConversionRate
pub fn from_hash_map_to_vec(
    data: HashMap<String, Decimal>,
    base: &str,
) -> Result<Vec<ConversionRate>> {
    let mut res: Vec<ConversionRate> = Vec::new();

    for (key, value) in data.into_iter() {
        res.push(ConversionRate {
            from: base.to_string(),
            to: key,
            rate: value,
        });
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::Path};

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::{
        commands::common::{conversion_rate::ConversionRate, create_or_update_file},
        config::Config,
    };

    #[test]
    fn from_hash_map_to_vec() {
        let mut hashmap = HashMap::new();

        let base = "EUR".to_string();
        let usd = ConversionRate {
            from: base.clone(),
            to: "USD".to_string(),
            rate: Decimal::new(108, 2),
        };
        hashmap.insert("USD".to_string(), Decimal::new(108, 2));

        let tbh = ConversionRate {
            from: base.clone(),
            to: "TBH".to_string(),
            rate: Decimal::new(32, 0),
        };

        hashmap.insert("TBH".to_string(), Decimal::new(32, 0));

        let res = super::from_hash_map_to_vec(hashmap, &base).unwrap();

        assert!(res.contains(&usd));
        assert!(res.contains(&tbh));
    }

    #[test]
    fn get_conversion_rate() {
        let base = "EUR".to_string();
        let usd = ConversionRate {
            from: base.clone(),
            to: "USD".to_string(),
            rate: Decimal::new(108, 2),
        };
        let tbh = ConversionRate {
            from: base.clone(),
            to: "TBH".to_string(),
            rate: Decimal::new(32, 0),
        };

        let data = vec![usd.clone(), tbh];

        let dirpath = "./temp/test/commands/common/conversion_rate/get_conversion_rate";

        std::fs::create_dir_all(dirpath).unwrap();

        let path = dirpath.to_string() + "/conversion_rate.tsv";

        create_or_update_file(&data, Path::new(&path)).unwrap();

        let config = Config {
            conversion_rates_file_path: path,
            base: "EUR".to_string(),
            ..Default::default()
        };

        let res = ConversionRate::get_conversion_rate(&config, "EUR", "USD");

        println!("{:?}", res);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), usd);

        std::fs::remove_dir_all(dirpath).unwrap();
    }

    #[test]
    fn get_conversion_rate2() {
        let base = "EUR".to_string();
        let usd = ConversionRate {
            from: base.clone(),
            to: "USD".to_string(),
            rate: Decimal::new(108, 2),
        };
        let tbh = ConversionRate {
            from: base.clone(),
            to: "TBH".to_string(),
            rate: Decimal::new(32, 0),
        };

        let data = vec![usd.clone(), tbh];

        let dirpath = "./temp/test/commands/common/conversion_rate/get_conversion_rate2";

        std::fs::create_dir_all(dirpath).unwrap();

        let path = dirpath.to_string() + "/conversion_rate.tsv";

        create_or_update_file(&data, Path::new(&path)).unwrap();

        let config = Config {
            conversion_rates_file_path: path,
            base: "EUR".to_string(),
            ..Default::default()
        };

        let res = ConversionRate::get_conversion_rate(&config, "USD", "EUR");

        println!("{:?}", res);

        let expected = ConversionRate {
            from: "USD".to_string(),
            to: "EUR".to_string(),
            rate: dec!(1) / usd.rate,
        };

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);

        std::fs::remove_dir_all(dirpath).unwrap();
    }
    #[test]
    fn get_conversion_rate3() {
        let base = "EUR".to_string();
        let usd = ConversionRate {
            from: base.clone(),
            to: "USD".to_string(),
            rate: Decimal::new(108, 2),
        };
        let tbh = ConversionRate {
            from: base.clone(),
            to: "TBH".to_string(),
            rate: Decimal::new(32, 0),
        };

        let data = vec![usd.clone(), tbh.clone()];

        let dirpath = "./temp/test/commands/common/conversion_rate/get_conversion_rate3";

        std::fs::create_dir_all(dirpath).unwrap();

        let path = dirpath.to_string() + "/conversion_rate.tsv";

        create_or_update_file(&data, Path::new(&path)).unwrap();

        let config = Config {
            conversion_rates_file_path: path,
            base: "EUR".to_string(),
            ..Default::default()
        };

        let res = ConversionRate::get_conversion_rate(&config, "USD", "TBH");

        println!("{:?}", res);

        let expected = ConversionRate {
            from: "USD".to_string(),
            to: "TBH".to_string(),
            rate: tbh.rate / usd.rate,
        };

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);

        std::fs::remove_dir_all(dirpath).unwrap();
    }
}
