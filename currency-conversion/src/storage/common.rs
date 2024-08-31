use std::{future::Future, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};


use super::{psql::PSQLStorageSettings, tsv::TSVStorageSettings};

/// Storage type available
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub enum StorageType {
    TSV(TSVStorageSettings),
    PSQL(PSQLStorageSettings),
}

/// Information about data
#[derive(Debug)]
pub enum DataInfo {
    Success(DataInfoSuccess),
    Error(DataInfoError),
}

#[derive(Debug)]
pub struct DataInfoSuccess {
    pub seconds_since_last_update: Duration,
    pub number_of_line: usize,
}

#[derive(Debug)]
pub struct DataInfoError {
    pub error: anyhow::Error,
}

/// Interface to store and retrieve data from storage
pub trait StorageManager<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    /// Update all data in storage
    fn update(&self, data: &[T]) -> impl Future<Output =  Result<()>>;

    /// Get all data from storage
    fn get_all(&self) -> impl Future<Output =  Result<Vec<T>>>;

    /// Get informations about data (last update, number, ...)
    fn get_data_info(&self) -> impl Future<Output =  Result<DataInfo>>;
}
