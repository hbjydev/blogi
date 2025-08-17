use async_trait::async_trait;

use crate::pg::PostgresDatastore;

#[async_trait]
pub trait ActorRepository {
}

impl ActorRepository for PostgresDatastore {

}
