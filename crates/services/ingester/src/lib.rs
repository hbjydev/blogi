use std::{collections::HashMap, sync::{Arc, Mutex}};

use anyhow::Result;
use async_trait::async_trait;
use blogi_errors::BlogiError;
use blogi_lexicons::moe::hayden::blogi::actor::Profile;
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

    // create your ingestors
    let mut ingestors: HashMap<String, Box<dyn LexiconIngestor + Send + Sync>> = HashMap::new();
    ingestors.insert(
        // your EXACT nsid
        "moe.hayden.blogi.actor.profile".to_string(),
        Box::new(ProfileIngestor),
    );

    // tracks the last message we've processed
    let cursor: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));

    // get channels
    let msg_rx = jetstream.get_msg_rx();
    let reconnect_tx = jetstream.get_reconnect_tx();

    // spawn a task to process messages from the queue.
    // this is a simple implementation, you can use a more complex one based on needs.
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

    Ok(
        jetstream
            .connect(cursor.clone())
            .await
            .map_err(|e| anyhow::anyhow!("error running ingest: {}", e))?
    )
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
