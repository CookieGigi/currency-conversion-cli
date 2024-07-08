use std::collections::HashMap;

use anyhow::Result;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::common::conversion_rate::ConversionRate;
use crate::storage::common::StorageManager;

use super::common::ErrorResponseAPI;

/// Update conversion rate files
pub async fn update_conversion_rates<T>(
    latest_endpoint_url: &str,
    api_key: &str,
    base: &str,
    conversion_rates_storage_manager: &T,
) -> Result<()>
where
    T: StorageManager<ConversionRate>,
{
    let url = latest_endpoint_url
        .replace("{api_key}", api_key)
        .replace("{base}", base);
    let data = get_conversion_rates(&url, base).await?;

    conversion_rates_storage_manager.update(&data).await?;

    Ok(())
}

/// Exchange rates API success response on latest endpoint
#[derive(Deserialize, Debug)]
struct SuccessLatestResponseAPI {
    //success: bool,
    //success: bool,
    //timestamp: i64,
    //base: String,
    //date: String,
    rates: HashMap<String, Decimal>,
}

/// Exchange rates API response on latest endpoint
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum LatestResponseAPI {
    Success(SuccessLatestResponseAPI),
    Fail(ErrorResponseAPI),
}

/// Get conversion rates for the base currency to others currencies from exchanges rates API
async fn get_conversion_rates(url: &str, base: &str) -> Result<Vec<ConversionRate>> {
    let response = reqwest::get(url).await?;

    tracing::debug!("{:?}", response);

    match response.json().await? {
        LatestResponseAPI::Success(s) => Ok(crate::common::conversion_rate::from_hash_map_to_vec(
            s.rates, base,
        )?),
        LatestResponseAPI::Fail(f) => Err(anyhow::anyhow!(
            "Call {} failed : {} - {}",
            url,
            f.error.code,
            f.error.message
        )),
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use httpmock::{Method::GET, MockServer};
    use rust_decimal_macros::dec;
    use serde_json::json;

    use crate::{common::conversion_rate::ConversionRate, storage::tsv::TSVStorageManager};

    fn setup(path: &str) {
        std::fs::create_dir_all(path).unwrap();
    }

    fn end(path: &str) {
        std::fs::remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn get_conversion_rates() {
        // param
        let api_key = "123";
        let base = "EUR";
        let server_response = json!({
            "success": true,
            "timestamp": 1519296206,
            "base": "EUR",
            "date": "2021-03-17",
            "rates": {
                "GBP": 0.72007,
                "JPY": 107.346001,
                "USD": 0.813399,
            }
        });
        let expected_usd = ConversionRate {
            from: base.to_string(),
            to: "USD".to_string(),
            rate: dec!(0.813399),
        };

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/test")
                .query_param("access_key", api_key)
                .query_param("base", base);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(server_response);
        });

        let response = super::get_conversion_rates(
            &server.url(format!("/test?access_key={api_key}&base={base}")),
            base,
        ).await;

        mock.assert();

        assert!(response.is_ok());
        assert!(response.unwrap().contains(&expected_usd));
    }

    #[tokio::test]
    async fn get_conversion_rates_fail() {
        // param
        let api_key = "123";
        let base = "EUR";
        let error_code = 104;
        let error_info =
            "Your monthly API request volume has been reached. Please upgrade your plan.";
        let server_response = json!({
          "success": false,
          "error": {
            "code": "104",
            "message": "Your monthly API request volume has been reached. Please upgrade your plan."
          }
        });

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/test")
                .query_param("access_key", api_key)
                .query_param("base", base);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(server_response);
        });

        let expected_error_message = format!(
            "Call {} failed : {} - {}",
            server.url(format!("/test?access_key={api_key}&base={base}")),
            error_code,
            error_info
        );

        let response = super::get_conversion_rates(
            &server.url(format!("/test?access_key={api_key}&base={base}")),
            base,
        ).await;

        mock.assert();

        assert!(response.is_err());
        assert_eq!(response.unwrap_err().to_string(), expected_error_message);
    }

    #[tokio::test]
    async fn update_conversion_rates() {
        // param
        let api_key = "123";
        let base = "EUR";
        let server_response = json!({
            "success": true,
            "timestamp": 1519296206,
            "base": "EUR",
            "date": "2021-03-17",
            "rates": {
                "GBP": 0.72007,
                "JPY": 107.346001,
                "USD": 0.813399,
            }
        });
        let dirpath = "./temp/test/commands/update/update_conversion/";

        setup(dirpath);

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/latest")
                .query_param("access_key", api_key)
                .query_param("base", base);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(server_response);
        });

        let mut file_path = PathBuf::new();
        file_path.push(dirpath.to_string() + "conversion_rates.tsv");

        let storage_manager = TSVStorageManager::build(file_path.clone());

        let response = super::update_conversion_rates(
            &server.url("/latest?access_key={api_key}&base={base}"),
            api_key,
            base,
            &storage_manager,
        ).await;

        mock.assert();

        println!("{:?}", response);
        assert!(response.is_ok());

        assert!(Path::new(&file_path).exists());

        end(dirpath);
    }
}
