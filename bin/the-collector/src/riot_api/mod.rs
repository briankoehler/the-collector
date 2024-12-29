use tokio::sync::mpsc::UnboundedSender;

pub mod account;
pub mod match_data;
pub mod match_ids;

pub trait Publish {
    type Input;
    type Output;

    async fn push(&self, data: Self::Input);
    async fn start(&self, publishing_channel: UnboundedSender<Self::Output>);
}
