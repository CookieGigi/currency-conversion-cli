use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use anyhow::{bail, Result};

use crate::list::list_data::ListDataItem;

/// Conversion Rates from a currency to another
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, PartialOrd, Eq, Ord,sqlx::FromRow)]
pub struct ConversionRate {
    pub from: String,
    pub to: String,
    pub rate: Decimal,
}

impl ListDataItem for ConversionRate {
    #[cfg(not(tarpaulin_include))]
    fn display_item(&self) -> String {
        format!("{} -> {} : {}", &self.from, &self.to, &self.rate)
    }
}

impl ConversionRate {
    /// Get conversion rate from a currency ([`from`]) to another ([`to`]) (from [`conversion_rates`] provided)
    pub fn get_conversion_rate(
        base: &str,
        conversion_rates: &Vec<ConversionRate>,
        from: &str,
        to: &str,
    ) -> Result<ConversionRate> {
        let res: ConversionRate;

        // if [`to`] == [`base`] then we search a rate which convert to [`from`] and return
        // inverted rate
        if to == base {
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
        // if [`from`] == [`base`] then we search a rate which convert to [`to`]
        } else if from == base {
            let mut search_iter = conversion_rates.iter().filter(|rate| rate.to == to);

            let search_result = search_iter.next();

            if search_result.is_none() {
                bail!("{to} symbols not found !");
            }
            res = search_result.unwrap().clone();
        // else we search to conversion rates :
        // - rates1 : from [`base`] to [`from`]
        // - rate2 : from [`base`] to [`to`]
        // and we return rate2/rate1
        } else {
            let rate_from =
                ConversionRate::get_conversion_rate(base, conversion_rates, base, from)?;
            let rate_to = ConversionRate::get_conversion_rate(base, conversion_rates, base, to)?;

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
    use std::collections::HashMap;

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::common::conversion_rate::ConversionRate;

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

        let res = ConversionRate::get_conversion_rate(&base, &data, "EUR", "USD");

        println!("{:?}", res);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), usd);
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

        let res = ConversionRate::get_conversion_rate(&base, &data, "USD", "EUR");

        println!("{:?}", res);

        let expected = ConversionRate {
            from: "USD".to_string(),
            to: "EUR".to_string(),
            rate: dec!(1) / usd.rate,
        };

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
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

        let res = ConversionRate::get_conversion_rate(&base, &data, "USD", "TBH");

        println!("{:?}", res);

        let expected = ConversionRate {
            from: "USD".to_string(),
            to: "TBH".to_string(),
            rate: tbh.rate / usd.rate,
        };

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
    }
    #[test]
    fn get_conversion_rate_unknow_from() {
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

        let res = ConversionRate::get_conversion_rate(&base, &data, "TEST", "EUR");

        println!("{:?}", res);

        assert!(res.is_err());
    }
    #[test]
    fn get_conversion_rate_unknown_to() {
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

        let res = ConversionRate::get_conversion_rate(&base, &data, "EUR", "TEST");

        println!("{:?}", res);

        assert!(res.is_err());
    }
}
