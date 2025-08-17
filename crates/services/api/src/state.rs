use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Box<dyn blogi_db::Datastore>>,
}
