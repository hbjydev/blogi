use std::{collections::HashMap, sync::{Arc, Mutex}};

use anyhow::Result;
use async_trait::async_trait;
use rocketman::{connection::JetstreamConnection, handler, ingestion::LexiconIngestor, options::JetstreamOptions, types::event::Event};
use serde_json::Value;
use tracing::info;

pub async fn start(_datastore: Box<dyn blogi_db::Datastore>) -> Result<()> {
    tracing::info!("ingester starting...");

    let opts = JetstreamOptions::builder()
        .wanted_collections(vec![
            "moe.hayden.blogi.actor.profile".to_string(),
        ])
        .build();

    let jetstream = JetstreamConnection::new(opts);

    let mut ingestors: HashMap<String, Box<dyn LexiconIngestor + Send + Sync>> = HashMap::new();
    ingestors.insert(
        "moe.hayden.blogi.actor.profile".to_string(),
        Box::new(ProfileIngestor),
    );

    let cursor: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));

    let msg_rx = jetstream.get_msg_rx();
    let reconnect_tx = jetstream.get_reconnect_tx();

    let c_cursor = cursor.clone();
    tokio::spawn(async move {
        while let Ok(message) = msg_rx.recv_async().await {
            if let Err(e) =
                handler::handle_message(message, &ingestors, reconnect_tx.clone(), c_cursor.clone())
                    .await
            {
                eprintln!("Error processing message: {}", e);
            };
        }
    });

    jetstream
        .connect(cursor.clone())
        .await
        .map_err(|e| anyhow::anyhow!("error running ingest: {}", e))
}

pub struct ProfileIngestor;

/// A cool ingestor implementation. Will just print the message. Does not do verification.
#[async_trait]
impl LexiconIngestor for ProfileIngestor {
    async fn ingest(&self, message: Event<Value>) -> Result<()> {
        info!("{:?}", message);
        // Process message for default lexicon.
        Ok(())
    }
}
