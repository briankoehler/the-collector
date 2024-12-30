use the_collector_db::DbHandler;
use crate::riot_api::match_data::MatchDataRequester;
use crate::riot_api::Publish;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error};

#[derive(Debug)]
pub struct MatchIdsHandler<P: Publish> {
    db_conn: Arc<DbHandler>,
    rx_channel: UnboundedReceiver<Vec<String>>,
    output: Arc<P>,
}

impl<P: Publish> MatchIdsHandler<P> {
    pub fn new(
        db_conn: Arc<DbHandler>,
        rx_channel: UnboundedReceiver<Vec<String>>,
        output: Arc<P>,
    ) -> Self {
        Self {
            db_conn,
            rx_channel,
            output,
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

            data.retain(|match_id| !db_matches.contains(match_id));
            self.output.push(data).await;
        }
    }
}
