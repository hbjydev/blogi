use async_trait::async_trait;
use blogi_errors::Success;
use sqlx::query;

// Drivers
pub mod pg;

// Repositories
pub mod actor;

// Cache
pub mod cache;

#[async_trait]
pub trait Datastore:
    actor::ActorRepository
    + Sync
    + Send
{
    fn boxed(self) -> Box<dyn Datastore>
    where
        Self: Sized + Sync + Send + 'static,
    {
        Box::new(self)
    }

    async fn ping(&self) -> Success;
}

#[async_trait]
impl Datastore for pg::PostgresDatastore {
    async fn ping(&self) -> Success {
        query!("SELECT 1 as one").fetch_one(&self.0).await?;
        Ok(())
    }
}
