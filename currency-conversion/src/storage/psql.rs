use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, query, Acquire, PgPool, Postgres, Transaction};

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

    async fn insert_one_symbol(&self, data: &Symbols, tx:&mut Transaction<'static, Postgres>) -> Result<()>{

        query!(r#"Insert Into symbols
                (id, "name", code)
                Values
                (
                gen_random_uuid(),
                $1,
                $2
                )"#,
                data.name,
                data.code).execute(&mut **tx).await?;
        Ok(())
    }

    async fn insert_one_conversion_rate(&self, data: &ConversionRate, tx:&mut Transaction<'static, Postgres>) -> Result<()>{

        query!(r#"Insert Into conversions_rates
                (id, "from", "to", rate)
                Values
                (
                gen_random_uuid(),
                $1,
                $2,
				$3
				)
                "#,
                data.from,
                data.to,
                data.rate
                ).execute(&mut **tx).await?;
        Ok(())
    }
}


impl StorageManager<Symbols> for PSQLStorageManager {
    async fn update(&self, data: &[Symbols]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete all from symbols
        sqlx::query("Delete from symbols").execute(&mut *tx).await?;

        for item in data {
            self.insert_one_symbol(item, &mut tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Symbols>> {
        let mut tx = self.pool.begin().await?;

        let res: Vec<Symbols> = sqlx::query_as::<_, Symbols>(r#"Select code, "name" from symbols"#).fetch_all(&mut *tx).await?;


        Ok(res)
    }

    async fn get_data_info(&self) -> Result<super::common::DataInfo> {
        Ok(super::common::DataInfo::Error(super::common::DataInfoError { error: anyhow!("Not implemented") }))

    }
}

impl StorageManager<ConversionRate> for PSQLStorageManager {
    async fn update(&self, data: &[ConversionRate]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete all from symbols
        sqlx::query("Delete from conversions_rates").execute(&mut *tx).await?;

        for item in data {
            self.insert_one_conversion_rate(item, &mut tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<ConversionRate>> {
        let mut tx = self.pool.begin().await?;

        let res: Vec<ConversionRate> = sqlx::query_as::<_, ConversionRate>(r#"Select "from", "to", rate from conversions_rates"#).fetch_all(&mut *tx).await?;


        Ok(res)
    }

    async fn get_data_info(&self) -> Result<super::common::DataInfo> {
        Ok(super::common::DataInfo::Error(super::common::DataInfoError { error: anyhow!("Not implemented") }))

    }
}
