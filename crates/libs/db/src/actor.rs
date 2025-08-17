use async_trait::async_trait;
use atrium_api::types::string::{AtIdentifier, Datetime, Did, Handle};
use blogi_errors::Result;
use blogi_lexicons::moe::hayden::blogi::actor::{defs::{ProfileViewDetailed, ProfileViewDetailedData}, profile};
use chrono::{DateTime, FixedOffset};
use sqlx::{prelude::FromRow, query_as};
use blogi_utils::resolve_identity;

use crate::pg::PostgresDatastore;

#[async_trait]
pub trait ActorRepository {
    async fn list_actors(&self, actors: Vec<AtIdentifier>) -> Result<Vec<ProfileViewDetailed>>;
    async fn get_actor(&self, actor: AtIdentifier) -> Result<Option<ProfileViewDetailed>>;
    async fn upsert_actor(&self, did: Did, record: &profile::RecordData) -> Result<()>;
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
            handle,
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
    async fn list_actors(&self, actors: Vec<AtIdentifier>) -> Result<Vec<ProfileViewDetailed>> {
        let actor_strs = actors.iter().map(|a| { a.as_ref().to_string() }).collect::<Vec<String>>();

        let data = query_as!(
            PgProfileRow,
            "SELECT
                did,
                handle,
                display_name,
                description,
                created_at,
                indexed_at
            FROM actors
            WHERE
                did = ANY($1::text[]) or handle = ANY($1::text[])",
            actor_strs.as_slice(),
        )
            .fetch_all(&self.0)
            .await?;

        let actors = data
            .into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<ProfileViewDetailed>>>()?;

        Ok(actors)
    }

    async fn get_actor(&self, actor: AtIdentifier) -> Result<Option<ProfileViewDetailed>> {
        let actors = self.list_actors(vec![actor]).await?;
        Ok(actors.into_iter().next())
    }

    async fn upsert_actor(&self, did: Did, profile: &profile::RecordData) -> Result<()> {
        let display_name = Some(profile.display_name.clone());
        let description = profile.description.clone();

        let ident = resolve_identity(&did, "https://public.api.bsky.app").await?;
        let handle_raw = ident.doc.also_known_as.first().to_owned().unwrap();
        let handle = if handle_raw.starts_with("at://") {
            handle_raw.clone()[5..].to_string()
        } else {
            handle_raw.clone()
        };

        let created_time = profile.created_at.clone().unwrap_or(Datetime::now());
        let chrono_time = created_time.as_ref();

        sqlx::query!(
            r#"
                INSERT INTO actors (did, handle, display_name, description, created_at, indexed_at)
                VALUES ($1, $2, $3, $4, $5, NOW())
                ON CONFLICT (did) DO UPDATE SET
                    handle = EXCLUDED.handle,
                    display_name = EXCLUDED.display_name,
                    description = EXCLUDED.description,
                    created_at = EXCLUDED.created_at
            "#,
            did.as_ref(),
            handle,
            display_name,
            description,
            chrono_time,
        )
            .execute(&self.0)
            .await?;

        Ok(())
    }
}
