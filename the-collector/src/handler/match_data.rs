use crate::{
    db::DbHandler,
    evaluator::MatchStatsEvaluator,
    ipc::{BytesSender, IntNotification},
};
use riven::models::match_v5::Match;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error};

#[derive(Debug)]
pub struct MatchDataHandler {
    db_conn: Arc<DbHandler>,
    evaluator: MatchStatsEvaluator,
    rx_channel: UnboundedReceiver<Match>,
    tx_channel: BytesSender<IntNotification>,
}

impl MatchDataHandler {
    pub fn new(
        db_conn: Arc<DbHandler>,
        rx_channel: UnboundedReceiver<Match>,
        tx_channel: BytesSender<IntNotification>,
    ) -> Self {
        Self {
            db_conn,
            rx_channel,
            tx_channel,
            evaluator: MatchStatsEvaluator::new(),
        }
    }

    /// Iterate on trying to receive data from [`Self::rx_channel`], and then
    // 1. Insert general data into DB
    // 2. Insert followed data into DB
    // 3. Evaluate stats if necessary
    #[tracing::instrument]
    pub async fn start(mut self) {
        loop {
            let data = self
                .rx_channel
                .recv()
                .await
                .expect("Receiving channel closed unexpectedly");
            debug!("Received Match data: {:?}", data.metadata.match_id);

            // Insert general match data into DB
            // TODO: Batch inserts and/or DB jobs queue
            if let Err(e) = self.db_conn.insert_match(&data).await {
                error!("Failed to insert match into database: {e:?}");
            }

            for puuid in &data.metadata.participants {
                // Insert followed info into DB
                if self.db_conn.get_summoner(puuid).await.is_ok() {
                    if let Err(e) = self.db_conn.insert_summoner_match(puuid, &data).await {
                        error!("Failed to insert summoner match data into database: {e:?}");
                    }
                }

                // Send through int evaluation
                // TODO: Refactor this
                let summoner_stats = data
                    .info
                    .participants
                    .iter()
                    .find(|p| p.puuid == *puuid)
                    .unwrap();
                if self.evaluator.is_int(&summoner_stats) {
                    // TODO: Generate random message
                    // TODO: Clean up this entire block
                    match self.db_conn.get_following_guilds(puuid).await {
                        Err(e) => error!("Failed to get followers of {puuid:?}: {e:?}"),
                        Ok(guilds) => {
                            for guild in guilds {
                                if let Err(e) = self
                                    .tx_channel
                                    .send(IntNotification {
                                        channel_id: guild
                                            .channel_id
                                            .expect("Channel ID to be set already") // TODO: Don't do this
                                            .try_into()
                                            .expect("Channel ID to fit in u64"),
                                        message: format!(
                                            "{} just died {} times",
                                            summoner_stats
                                                .riot_id_game_name
                                                .clone()
                                                .expect("Name to exist"),
                                            summoner_stats.deaths
                                        ),
                                    })
                                    .await
                                {
                                    error!("Failed to send int message: {e:?}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
