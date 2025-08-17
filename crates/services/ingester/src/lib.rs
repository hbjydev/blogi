use std::{collections::HashMap, sync::{Arc, Mutex}};

use anyhow::Result;
use async_trait::async_trait;
use atrium_api::types::string::Did;
use blogi_db::Datastore;
use rocketman::{connection::JetstreamConnection, handler, ingestion::LexiconIngestor, options::JetstreamOptions, types::event::Event};
use serde_json::Value;
use tracing::info;

pub async fn start(datastore: Box<dyn blogi_db::Datastore>) -> Result<()> {
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
        Box::new(ProfileIngestor(datastore)),
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

pub struct ProfileIngestor(pub Box<dyn Datastore>);

impl ProfileIngestor {
    /// Insert a profile into the datastore.
    #[tracing::instrument(skip(self, record))]
    pub async fn insert_profile(
        &self,
        did: Did,
        record: &blogi_lexicons::moe::hayden::blogi::actor::profile::RecordData,
    ) -> Result<()> {
        self.0
            .upsert_actor(did, record)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to insert profile: {}", e))
    }
}

/// A cool ingestor implementation. Will just print the message. Does not do verification.
#[async_trait]
impl LexiconIngestor for ProfileIngestor {
    #[tracing::instrument(skip(self))]
    async fn ingest(&self, message: Event<Value>) -> Result<()> {
        if let Some(commit) = &message.commit {
            if let Some(ref record) = commit.record {
                let record = serde_json::from_value::<
                    blogi_lexicons::moe::hayden::blogi::actor::profile::RecordData,
                >(record.clone())?;
                if let Some(ref commit) = message.commit {
                    if let Some(ref _cid) = commit.cid {
                        // TODO: verify cid
                        self.insert_profile(
                            Did::new(message.did)
                                .map_err(|e| anyhow::anyhow!("Failed to create Did: {}", e))?,
                            &record,
                        )
                        .await?;
                    }
                }
            } else {
                info!("commit has no record, assuming deletion");
            }
        } else {
            return Err(anyhow::anyhow!("Message has no commit"));
        }
        Ok(())
    }
}
