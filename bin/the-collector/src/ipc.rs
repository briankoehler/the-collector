use nng::Socket;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;

pub const INT_IPC_PATH: &str = "ipc:///tmp/int.ipc";

// TODO: Consider having messages constructed by the bot
#[derive(Debug, Serialize, Deserialize)]
pub struct IntNotification {
    pub channel_id: u64,
    pub message: String,
}

#[derive(Debug)]
pub struct BytesSender<T: Serialize + Send + Sync + 'static> {
    tx_int: Arc<Socket>,
    _phantom: PhantomData<T>,
}

impl<T: Serialize + Send + Sync + 'static> BytesSender<T> {
    pub fn new(stream: Arc<Socket>) -> Self {
        Self {
            tx_int: stream,
            _phantom: PhantomData,
        }
    }

    pub async fn send(&mut self, data: T) -> Result<(), Box<dyn std::error::Error>> {
        let tx_int = self.tx_int.clone();
        tokio::task::spawn_blocking(move || {
            tx_int
                .send(&bincode::serialize(&data).unwrap()).unwrap();
        }).await.unwrap();
        Ok(())
    }
}
