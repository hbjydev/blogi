use anyhow::Result;

pub async fn start(_datastore: Box<dyn blogi_db::Datastore>) -> Result<()> {
    tracing::info!("ingester starting...");
    Ok(())
}
