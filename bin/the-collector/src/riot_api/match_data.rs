use super::Publish;
use riven::{models::match_v5::Match, RiotApi};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tracing::debug;

/// Requester for fetching [`Match`] data from the Riot API given match IDs.
///
/// This should have its [`Publish::start`] method called within a Tokio task.
pub struct MatchDataRequester {
    riot_api: Arc<RiotApi>,
    match_queue: Mutex<VecDeque<String>>,
}

impl std::fmt::Debug for MatchDataRequester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchRequester")
            .field("match_queue", &self.match_queue)
            .finish()
    }
}

impl MatchDataRequester {
    pub fn new(riot_api: Arc<RiotApi>) -> Self {
        let match_queue = Mutex::new(VecDeque::new());
        Self {
            riot_api,
            match_queue,
        }
    }

    /// Fetch [`Match`] data from Riot API given a match ID.
    async fn get_match(&self, match_id: &str) -> Option<Match> {
        self.riot_api
            .match_v5()
            .get_match(riven::consts::RegionalRoute::AMERICAS, match_id)
            .await
            .unwrap()
    }
}

impl Publish for MatchDataRequester {
    type Input = Vec<String>;
    type Output = Match;

    /// Add a match ID to the queue.
    async fn push(&self, data: Self::Input) {
        let mut lock = self.match_queue.lock().await;
        lock.extend(data);
    }

    /// Loop the match IDs queue, fetching [`Match`] data for them and pushing the
    /// data to the provided publishing channel.
    #[tracing::instrument]
    async fn start(&self, publishing_channel: UnboundedSender<Self::Output>) {
        loop {
            let mut lock = self.match_queue.lock().await;
            if let Some(match_id) = lock.pop_front() {
                drop(lock);
                let match_data = self.get_match(&match_id).await.unwrap();
                debug!("Fetched match data: {:?}", match_data.metadata.match_id);
                publishing_channel.send(match_data).unwrap();
            }
        }
    }
}
