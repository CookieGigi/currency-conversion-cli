use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::common::{conversion_rate::ConversionRate, supported_symbols::Symbols};

use super::common::StorageManager;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct PSQLStorageSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl PSQLStorageSettings {

    pub fn get_url(&self) -> Result<String>{
        Ok(format!("postgres://{}:{}@{}:{}/{}", self.username, self.password, self.host, self.port, self.database_name))
    }
}

pub struct PSQLStorageManager {
    settings: PSQLStorageSettings,
    pool: PgPool, 
}

impl PSQLStorageManager {
    pub async fn from_settings(settings: PSQLStorageSettings) -> Result<PSQLStorageManager> {

        let url = settings.get_url()?;
        let pool = PgPoolOptions::new().connect(&url).await?;
        Ok(PSQLStorageManager { settings, pool })
    }
}

impl StorageManager<Symbols> for PSQLStorageManager {
    async fn update(&self, data: &[Symbols]) -> Result<()> {
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Symbols>> {
        Ok(vec![])
    }

    async fn get_data_info(&self) -> Result<super::common::DataInfo> {
        Ok(super::common::DataInfo::Error(super::common::DataInfoError { error: anyhow!("Not implemented") }))

    }
}

impl StorageManager<ConversionRate> for PSQLStorageManager {
    async fn update(&self, data: &[ConversionRate]) -> Result<()> {
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<ConversionRate>> {
        Ok(vec![])
    }

    async fn get_data_info(&self) -> Result<super::common::DataInfo> {
        Ok(super::common::DataInfo::Error(super::common::DataInfoError { error: anyhow!("Not implemented") }))

    }
}
