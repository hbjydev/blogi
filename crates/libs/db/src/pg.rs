use anyhow::Result;

pub struct PostgresDatastore(pub sqlx::pool::Pool<sqlx::Postgres>);

impl PostgresDatastore {
    pub async fn open(database_url: &str) -> Result<Self> {
        let pool = sqlx::Pool::connect_lazy(database_url)?;
        Ok(PostgresDatastore(pool))
    }
}
