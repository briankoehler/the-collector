use crate::error::IpcError;
use nng::{Error, Socket};
use serde::de::DeserializeOwned;
use std::{marker::PhantomData, sync::Arc};

#[derive(Debug)]
pub struct IpcSubscriber<T: DeserializeOwned> {
    socket: Arc<Socket>,
    _data_type: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> IpcSubscriber<T> {
    pub fn new(url: &str) -> Result<Self, Error> {
        let socket = Arc::new(Socket::new(nng::Protocol::Pull0)?);
        socket.listen(url)?;
        Ok(Self {
            socket,
            _data_type: PhantomData,
        })
    }

    pub async fn recv(&self) -> Result<T, IpcError> {
        let socket = self.socket.clone();
        let bytes = tokio::task::spawn_blocking(move || socket.recv()).await??;
        let data = bincode::deserialize(&bytes)?;
        Ok(data)
    }
}
