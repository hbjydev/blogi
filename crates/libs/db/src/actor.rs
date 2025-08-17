use async_trait::async_trait;
use atrium_api::types::string::{Datetime, Did, Handle};
use blogi_errors::Result;
use blogi_lexicons::moe::hayden::blogi::actor::defs::{ProfileViewDetailed, ProfileViewDetailedData};
use chrono::{DateTime, FixedOffset};
use sqlx::{prelude::FromRow, query_as};

use crate::pg::PostgresDatastore;

#[async_trait]
pub trait ActorRepository {
    async fn list_actors(&self) -> Result<Vec<ProfileViewDetailed>>;
}

#[derive(FromRow)]
struct PgProfileRow {
    pub did: String,
    pub handle: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub indexed_at: DateTime<FixedOffset>,
}

impl TryInto<ProfileViewDetailed> for PgProfileRow {
    type Error = blogi_errors::BlogiError;

    fn try_into(self) -> Result<ProfileViewDetailed> {
        let did = Did::new(self.did)
            .map_err(|e| anyhow::anyhow!(e))?;
        let handle = Handle::new(self.handle)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(ProfileViewDetailedData {
            did,
            handle: handle,
            display_name: Some(self.display_name),
            description: self.description,
            avatar: None, // Assuming avatar is not stored in this row
            banner: None, // Assuming banner is not stored in this row
            created_at: Some(Datetime::new(self.created_at)),
            posts_count: 0,
            indexed_at: Datetime::new(self.indexed_at),
        }.into())
    }
}

#[async_trait]
impl ActorRepository for PostgresDatastore {
    async fn list_actors(&self) -> Result<Vec<ProfileViewDetailed>> {
        let data = query_as!(
            PgProfileRow,
            "SELECT
                did,
                handle,
                display_name,
                description,
                created_at,
                indexed_at
            FROM actors",
        )
            .fetch_all(&self.0)
            .await?;

        let actors = data
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<ProfileViewDetailed>>>()?;

        Ok(actors)
    }
}
