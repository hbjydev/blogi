use std::time::Duration;

use async_trait::async_trait;
use blogi_errors::{Result, Success};
use chrono::{DateTime, FixedOffset};
use redis::TypedCommands;

#[async_trait]
pub trait Cache: Sync + Send {
    fn boxed(self) -> Box<dyn Cache>
    where
        Self: Sized + Sync + Send + 'static,
    {
        Box::new(self)
    }

    async fn ping(&self) -> Success;

    async fn get_string(&self, key: &str) -> Result<Option<String>>;
    async fn set_string(&self, key: &str, value: String, ttl: Duration) -> Success;

    async fn get_datetime(&self, key: &str) -> Result<Option<DateTime<FixedOffset>>>;
    async fn set_datetime(&self, key: &str, value: DateTime<FixedOffset>, ttl: Duration) -> Success;
}

pub struct RedisCache(pub redis::Client);

impl RedisCache {
    pub fn new(client: redis::Client) -> Self {
        RedisCache(client)
    }

    pub fn open(addr: &str) -> Result<Self> {
        let client = redis::Client::open(addr)?;
        Ok(RedisCache(client))
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn ping(&self) -> Success {
        let mut con = self.0.get_connection()?;
        con.ping()?;
        Ok(())
    }

    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        let mut con = self.0.get_connection()?;
        Ok(con.get(key)?)
    }

    async fn set_string(&self, key: &str, value: String, ttl: Duration) -> Success {
        let mut con = self.0.get_connection()?;
        Ok(con.set_ex(key, value, ttl.as_secs())?)
    }

    async fn get_datetime(&self, key: &str) -> Result<Option<DateTime<FixedOffset>>> {
        let mut con = self.0.get_connection()?;
        let value = con.get(key)?;
        match value {
            Some(s) => Ok(Some(DateTime::parse_from_rfc3339(&s)?)),
            None => Ok(None),
        }
    }

    async fn set_datetime(&self, key: &str, value: DateTime<FixedOffset>, ttl: Duration) -> Success {
        let mut con = self.0.get_connection()?;
        Ok(con.set_ex(key, value.to_rfc3339(), ttl.as_secs())?)
    }
}
