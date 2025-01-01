use nng::{Error, Socket};
use serde::Serialize;
use std::{marker::PhantomData, sync::Arc};

#[derive(Debug)]
pub struct IpcPublisher<T: Serialize> {
    socket: Arc<Socket>,
    _data_type: PhantomData<T>,
}

impl<T: Serialize + Send + Sync> IpcPublisher<T> {
    pub fn new(url: &str) -> Result<Self, Error> {
        let socket = Arc::new(Socket::new(nng::Protocol::Push0)?);
        socket.dial(url)?;
        Ok(Self {
            socket,
            _data_type: PhantomData,
        })
    }

    // TODO: Return specific error types
    pub async fn publish(&self, data: T) -> anyhow::Result<()> {
        let bytes = bincode::serialize(&data)?;
        let socket = self.socket.clone();
        tokio::task::spawn_blocking(move || socket.send(&bytes).map_err(|err| err.1)).await??;
        Ok(())
    }
}
