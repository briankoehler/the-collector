use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error(transparent)]
    SerializationError(#[from] bincode::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    NngError(#[from] nng::Error),
}
