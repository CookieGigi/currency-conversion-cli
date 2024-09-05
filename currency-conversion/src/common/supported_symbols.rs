use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::list::list_data::ListDataItem;

/// Symbols of a currency
#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, sqlx::FromRow)]
pub struct Symbols {
    pub code: String,
    pub name: String,
}

impl ListDataItem for Symbols {
    #[cfg(not(tarpaulin_include))]
    fn display_item(&self) -> String {
        format!("{} : {}", &self.code, &self.name)
    }
}

/// Convert a hahmap (key : code, value : name) to a vec of Symbols
pub fn from_hash_map_to_vec(data: HashMap<String, String>) -> Result<Vec<Symbols>> {
    let mut res: Vec<Symbols> = Vec::new();

    for (key, value) in data.into_iter() {
        res.push(Symbols {
            code: key.clone(),
            name: value.clone(),
        });
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::common::supported_symbols::Symbols;

    #[test]
    fn from_hash_map_to_vec() {
        let mut hashmap = HashMap::new();

        let eur = Symbols {
            code: "EUR".to_string(),
            name: "Euro".to_string(),
        };
        hashmap.insert("EUR".to_string(), "Euro".to_string());

        let tbh = Symbols {
            code: "TBH".to_string(),
            name: "Thai Baht".to_string(),
        };

        hashmap.insert("TBH".to_string(), "Thai Baht".to_string());

        let res = super::from_hash_map_to_vec(hashmap).unwrap();

        assert!(res.contains(&eur));
        assert!(res.contains(&tbh));
    }
}
