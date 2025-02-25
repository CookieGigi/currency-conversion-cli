use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, query, query_as, PgPool, Postgres, Transaction};
use sqlx::postgres::types::PgInterval;
use sqlx::Row;


use crate::common::{conversion_rate::ConversionRate, supported_symbols::Symbols};

use super::common::{DataInfoSuccess, StorageManager};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct PSQLStorageSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[cfg(not(tarpaulin_include))]
impl PSQLStorageSettings {
    pub fn get_url(&self) -> Result<String> {
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        ))
    }
}

pub struct PSQLStorageManager {
    //settings: PSQLStorageSettings,
    pool: PgPool,
}

impl PSQLStorageManager {
    #[cfg(not(tarpaulin_include))]
    pub async fn from_settings(settings: PSQLStorageSettings) -> Result<PSQLStorageManager> {
        let url = settings.get_url()?;
        let pool = PgPoolOptions::new().connect(&url).await?;
        Ok(PSQLStorageManager { pool })
    }

    async fn insert_one_symbol(
        &self,
        data: &Symbols,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        query!(
            r#"Insert Into symbols
                (id, "name", code)
                Values
                (
                gen_random_uuid(),
                $1,
                $2
                )"#,
            data.name,
            data.code
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn insert_one_conversion_rate(
        &self,
        data: &ConversionRate,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        query!(
            r#"Insert Into conversions_rates
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
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn update_data_info(
        &self,
        data_name: &str,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        query!(
            r#"INSERT INTO data_info 
                (data_name, last_update_date)
                Values
                ($1, NOW())
                ON CONFLICT (data_name)
                DO 
                UPDATE Set last_update_date = NOW();"#,
            data_name
        )
        .execute(&mut **tx)
        .await?;
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

        self.update_data_info("symbols", &mut tx).await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Symbols>> {
        let mut tx = self.pool.begin().await?;

        let res: Vec<Symbols> = sqlx::query_as::<_, Symbols>(r#"Select code, "name" from symbols"#)
            .fetch_all(&mut *tx)
            .await?;

        Ok(res)
    }

    async fn get_data_info(&self) -> Result<super::common::DataInfo> {
        let mut tx = self.pool.begin().await?;

        Ok(super::common::DataInfo::Success(
            query_as(
                r#"Select NOW() - last_update_date as seconds_since_last_update, (Select count(*) From symbols) as number_of_line  from data_info where data_name = 'symbols'"#,
                
            )
            .fetch_one(&mut *tx)
            .await?,
        ))
    }
}

impl StorageManager<ConversionRate> for PSQLStorageManager {
    async fn update(&self, data: &[ConversionRate]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete all from symbols
        sqlx::query("Delete from conversions_rates")
            .execute(&mut *tx)
            .await?;

        for item in data {
            self.insert_one_conversion_rate(item, &mut tx).await?;
        }

        self.update_data_info("conversions_rates", &mut tx).await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<ConversionRate>> {
        let mut tx = self.pool.begin().await?;

        let res: Vec<ConversionRate> = sqlx::query_as::<_, ConversionRate>(
            r#"Select "from", "to", rate from conversions_rates"#,
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(res)
    }

    async fn get_data_info(&self) -> Result<super::common::DataInfo> {
        let mut tx = self.pool.begin().await?;

        Ok(super::common::DataInfo::Success(
            query_as(
                r#"Select NOW() - last_update_date as seconds_since_last_update, (Select count(*) From conversions_rates) as number_of_line  from data_info where data_name = 'conversions_rates'"#,
                
            )
            .fetch_one(&mut *tx)
            .await?,
        ))
    }
}


impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for DataInfoSuccess {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let seconds_since_last_update: PgInterval = row.try_get("seconds_since_last_update")?;
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
 
#[cfg(test)]
mod test {


    use rust_decimal::Decimal;
    use sqlx::{ query, PgPool};

    use crate::{common::{conversion_rate::ConversionRate, supported_symbols::Symbols}, storage::common::{DataInfo, StorageManager}};

    #[sqlx::test]
    async fn insert_one_symbol(pool: PgPool) -> sqlx::Result<()>{

      // init storage manager
      let sm = super::PSQLStorageManager{pool: pool.clone()};

      // data test
      let data = Symbols{ code : "EUR".to_string(), name: "Euro".to_string()};

      // execution 
      let mut tx = pool.begin().await.unwrap();
      let res = sm.insert_one_symbol(&data, &mut tx).await;
      assert!(res.is_ok()); 
      tx.commit().await.unwrap();

      // verify in database
      let data_res = StorageManager::<Symbols>::get_all(&sm).await.unwrap();

      assert_eq!(data_res.len(),  1);
      assert_eq!(data_res[0], data);

      Ok(())
    }

    #[sqlx::test]
    async fn insert_one_conversion_rate(pool: PgPool) -> sqlx::Result<()>{

      // init storage manager
      let sm = super::PSQLStorageManager{pool: pool.clone()};

      // data test
      let data = ConversionRate{from: "EUR".to_string(), to: "USD".to_string(), rate: Decimal::new(81, 2)   };

      // execution 
      let mut tx = pool.begin().await.unwrap();
      let res = sm.insert_one_conversion_rate(&data, &mut tx).await;
      assert!(res.is_ok()); 
      tx.commit().await.unwrap();

      // verify in database
      let data_res = StorageManager::<ConversionRate>::get_all(&sm).await.unwrap();

      assert_eq!(data_res.len(),  1);
      assert_eq!(data_res[0], data);

      Ok(())
    }

    #[sqlx::test]
    async fn update_data_info(pool: PgPool) -> sqlx::Result<()>{

      // init storage manager
      let sm = super::PSQLStorageManager{pool: pool.clone()};

      // execution 
      let mut tx = pool.begin().await.unwrap();
      let res = sm.update_data_info("test", &mut tx).await;
      assert!(res.is_ok()); 
      tx.commit().await.unwrap();

      Ok(())
    }

    #[sqlx::test]
    async fn get_data_info_success(pool: PgPool) {
      // data preparation
    }
}
