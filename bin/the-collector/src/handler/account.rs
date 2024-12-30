use the_collector_db::DbHandler;
use riven::models::account_v1::Account;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error};

#[derive(Debug)]
pub struct AccountHandler {
    db_conn: Arc<DbHandler>,
    rx_channel: UnboundedReceiver<Account>,
}

impl AccountHandler {
    pub fn new(db_conn: Arc<DbHandler>, rx_channel: UnboundedReceiver<Account>) -> Self {
        Self {
            db_conn,
            rx_channel,
        }
    }

    /// Iterate on trying to receive data from [`Self::rx_channel`], and handle
    /// by inserting the data into the DB.
    #[tracing::instrument]
    pub async fn start(mut self) {
        loop {
            let data = self
                .rx_channel
                .recv()
                .await
                .expect("Receiving channel closed unexpectedly");
            debug!("Received Account data: {data:?}");

            if let Err(e) = self.db_conn.insert_summoner(data).await {
                error!("Failed to insert summoner to database: {e:?}");
            }
        }
    }
}
