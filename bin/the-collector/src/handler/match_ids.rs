use crate::riot_api::match_data::MatchDataRequester;
use crate::riot_api::Publish;
use circular_queue::CircularQueue;
use std::sync::Arc;
use the_collector_db::DbHandler;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error};

const CACHE_SIZE: usize = 100;

#[derive(Debug)]
pub struct MatchIdsHandler<P: Publish> {
    db_conn: Arc<DbHandler>,
    rx_channel: UnboundedReceiver<Vec<String>>,
    output: Arc<P>,
    // TODO: Consider removing the cache
    cache: CircularQueue<String>,
}

impl<P: Publish> MatchIdsHandler<P> {
    pub fn new(
        db_conn: Arc<DbHandler>,
        rx_channel: UnboundedReceiver<Vec<String>>,
        output: Arc<P>,
    ) -> Self {
        let cache = CircularQueue::with_capacity(CACHE_SIZE);
        Self {
            db_conn,
            rx_channel,
            output,
            cache,
        }
    }
}

impl MatchIdsHandler<MatchDataRequester> {
    /// Iterate on trying to receive data from [`Self::rx_channel`], and push the
    /// data to [`Self::output`].
    #[tracing::instrument]
    pub async fn start(mut self) {
        loop {
            let mut data = self
                .rx_channel
                .recv()
                .await
                .expect("Receiving channel closed unexpectedly");
            debug!("Received Matches data: {data:?}");

            let Ok(matches) = self.db_conn.get_matches(&data).await else {
                error!("Error getting existing match entries");
                continue;
            };
            debug!(
                "Found {:?} of {:?} matches already in database",
                matches.len(),
                data.len()
            );
            let db_matches: Vec<String> = matches.into_iter().map(|m| m.id).collect();

            // Remove games that are already in the cache, or are in the database
            data.retain(|match_id| !self.cache.iter().any(|cache_id| cache_id == match_id));
            data.retain(|match_id| !db_matches.contains(match_id));

            // Add match IDs to cache and push out
            for match_id in &data {
                self.cache.push(match_id.clone());
            }
            self.output.push(data).await;
        }
    }
}
