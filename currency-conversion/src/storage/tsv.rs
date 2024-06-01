use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::common::{conversion_rate::ConversionRate, supported_symbols::Symbols};

use super::common::{DataInfo, DataInfoError, DataInfoSuccess, StorageManager};

use anyhow::{anyhow, Result};

/// Store data in tsv file
pub struct TSVStorageManager {
    settings: TSVStorageSettings,
}

/// Settings for TSVStorageManager
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct TSVStorageSettings {
    pub file_path: String,
}

impl TSVStorageManager {
    /// Build a TSVStorageManager with the path to the file where are store
    pub fn build(path: String) -> TSVStorageManager {
        TSVStorageManager {
            settings: TSVStorageSettings { file_path: path },
        }
    }

    /// Build a TSVStorageManager from settings
    pub fn from_settings(settings: TSVStorageSettings) -> TSVStorageManager {
        TSVStorageManager { settings }
    }
}

impl StorageManager<ConversionRate> for TSVStorageManager {
    fn update(&self, data: &[ConversionRate]) -> anyhow::Result<()> {
        create_or_update_file::<ConversionRate>(data, Path::new(&self.settings.file_path))
    }

    fn get_all(&self) -> anyhow::Result<Vec<ConversionRate>> {
        load_data(Path::new(&self.settings.file_path))
    }

    fn get_data_info(&self) -> Result<DataInfo> {
        get_data_info::<ConversionRate>(&self.settings.file_path)
    }
}

impl StorageManager<Symbols> for TSVStorageManager {
    fn update(&self, data: &[Symbols]) -> anyhow::Result<()> {
        create_or_update_file::<Symbols>(data, Path::new(&self.settings.file_path))
    }

    fn get_all(&self) -> anyhow::Result<Vec<Symbols>> {
        load_data(Path::new(&self.settings.file_path))
    }

    fn get_data_info(&self) -> Result<DataInfo> {
        get_data_info::<Symbols>(&self.settings.file_path)
    }
}

/// Write a Vec in a tsv file                                                                                
fn create_or_update_file<T>(data: &[T], path: &Path) -> Result<()>
where
    T: Serialize,
{
    tracing::info!("Updating {:?}", path);
    let mut wrt = csv::WriterBuilder::new().delimiter(b'\t').from_path(path)?;

    for row in data {
        wrt.serialize(row)?;
    }

    tracing::info!("Updated {:?}", path);

    Ok(())
}

/// Load data from tsv file
fn load_data<'a, T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    tracing::info!("Reading {:?}", path);
    let mut csv_rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;

    let mut res = Vec::new();

    for line in csv_rdr.deserialize() {
        res.push(line?);
    }
    tracing::info!("Readed {:?}", path);

    Ok(res)
}

/// Get information about data                                                                                               
fn get_data_info<T>(path: &str) -> Result<DataInfo>
where
    T: for<'de> Deserialize<'de>,
{
    let file_metadata = std::fs::metadata(path);

    match file_metadata {
        Ok(metadata) => match load_data::<T>(Path::new(path)) {
            Ok(data) => Ok(DataInfo::Success(DataInfoSuccess {
                seconds_since_last_update: metadata.modified()?.elapsed()?,
                number_of_line: data.len(),
            })),
            #[cfg(not(tarpaulin_include))]
            Err(e) => Ok(DataInfo::Error(DataInfoError { error: e })),
        },
        Err(e) => Ok(DataInfo::Error(DataInfoError { error: anyhow!(e) })),
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use serde::{Deserialize, Serialize};

    use crate::storage::common::DataInfo;

    #[derive(Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        code: u8,
        name: String,
    }

    #[test]
    fn create_or_update_file() {
        let dirpath = "./temp/test/storage/tsv/create_or_update_file";

        std::fs::create_dir_all(dirpath).unwrap();

        let data = vec![
            TestStruct {
                code: 1,
                name: "1".to_string(),
            },
            TestStruct {
                code: 2,
                name: "2".to_string(),
            },
        ];

        let path = dirpath.to_string() + "/test.tsv";

        assert!(super::create_or_update_file(&data, Path::new(&path)).is_ok());
        assert!(Path::new(&path).exists());

        std::fs::remove_dir_all(dirpath).unwrap();
    }

    #[test]
    fn load_data() {
        let data: Vec<TestStruct> = vec![
            TestStruct {
                code: 1,
                name: "1".to_string(),
            },
            TestStruct {
                code: 2,
                name: "2".to_string(),
            },
        ];

        let dirpath = "./temp/test/commands/update/common/load_data";

        std::fs::create_dir_all(dirpath).unwrap();

        let path = dirpath.to_string() + "/test2.tsv";

        super::create_or_update_file(&data, Path::new(&path)).unwrap();

        let res = super::load_data::<TestStruct>(Path::new(&path));

        assert!(res.is_ok());
        assert!(res.unwrap().contains(&data[0]));

        std::fs::remove_dir_all(dirpath).unwrap();
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestData {
        code: u8,
        name: String,
    }

    #[test]
    fn get_data_info_success() {
        let data = vec![
            TestData {
                code: 1,
                name: "1".to_string(),
            },
            TestData {
                code: 2,
                name: "2".to_string(),
            },
        ];

        let dirpath = "./temp/test/commands/info/common/get_data_info_success";

        std::fs::create_dir_all(dirpath).unwrap();

        let path = dirpath.to_string() + "/test.tsv";

        super::create_or_update_file(&data, Path::new(&path)).unwrap();

        let res = super::get_data_info::<TestData>(&path);

        assert!(res.is_ok());

        assert!(matches!(res.unwrap(), DataInfo::Success(..)));

        std::fs::remove_dir_all(dirpath).unwrap();
    }

    #[test]
    fn get_data_info_error() {
        let dirpath = "./temp/test/commands/info/common/get_data_info_error";

        std::fs::create_dir_all(dirpath).unwrap();

        let path = dirpath.to_string() + "/test.tsv";

        let res = super::get_data_info::<TestData>(&path);

        assert!(res.is_ok());

        assert!(matches!(res.unwrap(), DataInfo::Error(..)));

        std::fs::remove_dir_all(dirpath).unwrap();
    }
}
