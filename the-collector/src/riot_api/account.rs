use super::Publish;
use riven::consts::RegionalRoute::AMERICAS;
use riven::{models::account_v1::Account, RiotApi};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug)]
pub struct UsernameAndTag(pub String, pub String);

/// Handler for retrieving [`Account`] information from Riot API, given a
/// [`UsernameAndTag`]. [`Publish::start`] should be called within
/// its own Tokio task.
pub struct AccountRequester {
    riot_api: Arc<RiotApi>,
    account_queue: Mutex<VecDeque<UsernameAndTag>>,
}

impl std::fmt::Debug for AccountRequester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountRequester")
            .field("account_queue", &self.account_queue)
            .finish()
    }
}

impl AccountRequester {
    pub fn new(riot_api: Arc<RiotApi>) -> Self {
        let account_queue = tokio::sync::Mutex::new(VecDeque::new());
        Self {
            riot_api,
            account_queue,
        }
    }

    /// Retrieve an [`Account`] from a [`UsernameAndTag`]. If no account information
    /// is found, return [`None`].
    async fn get_account(&self, account_info: &UsernameAndTag) -> Option<Account> {
        self.riot_api
            .account_v1()
            .get_by_riot_id(AMERICAS, &account_info.0, &account_info.1)
            .await
            .unwrap()
    }
}

impl Publish for AccountRequester {
    type Input = UsernameAndTag;
    type Output = Account;

    /// Push a [`UsernameAndTag`] onto the queue.
    async fn push(&self, data: Self::Input) {
        let mut lock = self.account_queue.lock().await;
        lock.push_back(data);
    }

    /// Loop the queue of [`UsernameAndTag`], and fetch [`Account`] data from Riot's API.
    ///
    /// Fetched data is sent to the provided publishing channel.
    #[tracing::instrument]
    async fn start(&self, publishing_channel: tokio::sync::mpsc::UnboundedSender<Self::Output>) {
        loop {
            let mut lock = self.account_queue.lock().await;
            if let Some(user_data) = lock.pop_front() {
                drop(lock);
                let account = self.get_account(&user_data).await.unwrap();
                debug!("Fetched account: {account:?}");
                publishing_channel.send(account).unwrap();
            }
        }
    }
}
