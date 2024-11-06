use std::{future::Future, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{psql::PSQLStorageSettings, tsv::TSVStorageSettings};
use sqlx::postgres::types::PgInterval;
use sqlx::Row;

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

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for DataInfoSuccess {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let seconds_since_last_update: PgInterval = row.try_get("seconds_since_last_update")?;
        tracing::debug!("{:?}", seconds_since_last_update);
        let seconds_since_last_update = Duration::new(
            (seconds_since_last_update.months * 2678400 + seconds_since_last_update.days * 86400)
                as u64,
            seconds_since_last_update.microseconds as u32,
        );
        let number_of_line: i64 = row.try_get("number_of_line")?;
        // Impossible to have a bad value
        let number_of_line: usize = number_of_line.try_into().unwrap();
        Ok(Self {
            seconds_since_last_update,
            number_of_line,
        })
    }
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
    fn update(&self, data: &[T]) -> impl Future<Output = Result<()>>;

    /// Get all data from storage
    fn get_all(&self) -> impl Future<Output = Result<Vec<T>>>;

    /// Get informations about data (last update, number, ...)
    fn get_data_info(&self) -> impl Future<Output = Result<DataInfo>>;
}
