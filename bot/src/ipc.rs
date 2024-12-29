use nng::Socket;
use serde::de::DeserializeOwned;
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
pub struct BytesReceiver<T: DeserializeOwned> {
    rx_int: Arc<Socket>,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> BytesReceiver<T> {
    pub fn new(stream: Arc<Socket>) -> Self {
        Self {
            rx_int: stream,
            _phantom: PhantomData,
        }
    }

    pub async fn recv(&mut self) -> Result<T, Box<dyn std::error::Error>> {
        let rx_int = self.rx_int.clone();
        let msg = tokio::task::spawn_blocking(move || rx_int.recv().unwrap())
            .await
            .unwrap();
        let notification = bincode::deserialize(&msg).unwrap();
        Ok(notification)
    }
}
