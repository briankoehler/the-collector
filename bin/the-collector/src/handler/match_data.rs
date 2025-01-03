use riven::models::match_v5::Match;
use std::sync::Arc;
use the_collector_db::DbHandler;
use the_collector_ipc::{r#pub::IpcPublisher, SummonerMatchQuery};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct MatchDataHandler {
    db_conn: Arc<DbHandler>,
    rx_channel: UnboundedReceiver<Match>,
    publisher: IpcPublisher<SummonerMatchQuery>,
}

impl MatchDataHandler {
    pub fn new(
        db_conn: Arc<DbHandler>,
        rx_channel: UnboundedReceiver<Match>,
        publisher: IpcPublisher<SummonerMatchQuery>,
    ) -> Self {
        Self {
            db_conn,
            rx_channel,
            publisher,
        }
    }

    /// Iterate on trying to receive data from [`Self::rx_channel`], and then
    // 1. Insert general data into DB
    // 2. Insert followed data into DB
    // 3. Send match ID to
    #[tracing::instrument]
    pub async fn start(mut self) {
        loop {
            match self.run().await {
                Ok(count) => info!("Inserted {count} summoner matches"),
                Err(e) => error!("Match Data Handler error: {e:?}"),
            }
        }
    }

    async fn run(&mut self) -> anyhow::Result<u8> {
        // Receive data
        let data = self
            .rx_channel
            .recv()
            .await
            .expect("Receiving channel closed unexpectedly");
        debug!("Received Match data: {:?}", data.metadata.match_id);

        // Insert general match data into DB
        // TODO: Batch inserts and/or DB jobs queue
        if let Err(e) = self.db_conn.insert_match(&data).await {
            // TODO: Retry
            anyhow::bail!("Failed to insert match into database: {e:?}");
        }

        // Insert followed info into DB
        let mut count = 0;
        for puuid in &data.metadata.participants {
            if self.db_conn.get_summoner(puuid).await?.is_none() {
                continue;
            }

            if let Err(e) = self.db_conn.insert_summoner_match(puuid, &data).await {
                error!("Failed to insert summoner match data into database: {e:?}");
                continue;
            }

            // TODO: Avoid cloning?
            let message = SummonerMatchQuery {
                puuid: puuid.clone(),
                match_id: data.metadata.match_id.clone(),
            };
            debug!("Sending match query: {message:?}");
            self.publisher.publish(message).await?;
            count += 1;
        }

        Ok(count)
    }
}
