use std::collections::HashMap;

use crate::common::supported_symbols::Symbols;
use crate::storage::common::StorageManager;

use anyhow::Result;
use serde::Deserialize;

use super::common::ErrorResponseAPI;

/// Update supported symbols file
pub fn update_symbols<T>(
    symbols_endpoint_url: &str,
    api_key: &str,
    symbols_storage_manager: &T,
) -> Result<()>
where
    T: StorageManager<Symbols>,
{
    let url = symbols_endpoint_url.replace("{api_key}", api_key);
    let symbols = get_supported_symbols(&url)?;

    tracing::debug!("{:?}", &symbols);
    tracing::info!("{} Symbols updated", symbols.len());

    symbols_storage_manager.update(&symbols)?;

    Ok(())
}

/// Exchange rates API success response on symbols endpoint
#[derive(Deserialize, Debug)]
struct SuccessSymbolResponseAPI {
    //success: bool,
    symbols: HashMap<String, String>,
}

/// Exchange rates API response on symbols endpoint
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SymbolResponseAPI {
    Success(SuccessSymbolResponseAPI),
    Fail(ErrorResponseAPI),
}

/// Get supported symbols from exchanges rates API
fn get_supported_symbols(url: &str) -> Result<Vec<Symbols>> {
    let response = reqwest::blocking::get(url)?;

    match response.json()? {
        SymbolResponseAPI::Success(s) => Ok(
            crate::common::supported_symbols::from_hash_map_to_vec(s.symbols)?,
        ),
        SymbolResponseAPI::Fail(f) => Err(anyhow::anyhow!(
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
    use serde_json::json;

    use crate::{common::supported_symbols::Symbols, storage::tsv::TSVStorageManager};

    fn setup(path: &str) {
        std::fs::create_dir_all(path).unwrap();
    }

    fn end(path: &str) {
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn get_supported_symbols() {
        // param
        let api_key = "123";

        let server_response = json!(
        {
          "success": true,
          "symbols": {
            "AED": "United Arab Emirates Dirham",
            "AFN": "Afghan Afghani",
            "ALL": "Albanian Lek",
            "AMD": "Armenian Dram",
            }
        }
        );
        let expected_all = Symbols {
            code: "ALL".to_string(),
            name: "Albanian Lek".to_string(),
        };

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/symbols")
                .query_param("access_key", api_key);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(server_response);
        });

        let response =
            super::get_supported_symbols(&server.url(format!("/symbols?access_key={api_key}")));

        mock.assert();

        assert!(response.is_ok());
        assert!(response.unwrap().contains(&expected_all));
    }

    #[test]
    fn get_supported_symbols_fail() {
        // param
        let api_key = "123";
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
                .query_param("access_key", api_key);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(server_response);
        });

        let expected_error_message = format!(
            "Call {} failed : {} - {}",
            server.url(format!("/test?access_key={api_key}")),
            error_code,
            error_info
        );

        let response =
            super::get_supported_symbols(&server.url(format!("/test?access_key={api_key}")));

        mock.assert();

        assert!(response.is_err());
        assert_eq!(response.unwrap_err().to_string(), expected_error_message);
    }

    #[test]
    fn update_symbols() {
        // param
        let api_key = "123";
        let server_response = json!(
        {
          "success": true,
          "symbols": {
            "AED": "United Arab Emirates Dirham",
            "AFN": "Afghan Afghani",
            "ALL": "Albanian Lek",
            "AMD": "Armenian Dram",
            }
        }
        );
        let dirpath = "./temp/test/commands/update/update_symbols/";

        setup(dirpath);

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/test")
                .query_param("access_key", api_key);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(server_response);
        });

        let mut file_path = PathBuf::new();

        file_path.push(dirpath.to_string() + "symbols.tsv");

        let storage_manager = TSVStorageManager::build(file_path.clone());

        let response = super::update_symbols(
            &server.url("/test?access_key={api_key}"),
            api_key,
            &storage_manager,
        );

        mock.assert();

        println!("{:?}", response);
        assert!(response.is_ok());

        assert!(Path::new(&file_path).exists());

        end(dirpath);
    }
}
