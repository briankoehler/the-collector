use super::Publish;
use riven::consts::RegionalRoute::AMERICAS;
use riven::RiotApi;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tracing::debug;

// Max value that Riot API accepts for getting match IDs
const MAX_MATCHES: i32 = 100;

#[derive(Debug)]
pub struct GetMatchIdsQuery {
    pub puuid: String,
    pub start_time: Option<i64>,
    pub count: Option<i32>,
}

/// Requester for fetching Match IDs from the Riot API given
/// a [`GetMatchesQuery`].
pub struct MatchIdsRequester {
    riot_api: Arc<RiotApi>,
    matches_queue: Mutex<VecDeque<GetMatchIdsQuery>>,
}

impl std::fmt::Debug for MatchIdsRequester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchesRequester")
            .field("matches_queue", &self.matches_queue)
            .finish()
    }
}

impl MatchIdsRequester {
    pub fn new(riot_api: Arc<RiotApi>) -> Self {
        let matches_queue = Mutex::new(VecDeque::new());
        Self {
            riot_api,
            matches_queue,
        }
    }

    /// Get matches from Riot API given a [`GetMatchesQuery`].
    async fn get_matches(&self, query: &GetMatchIdsQuery) -> Vec<String> {
        self.riot_api
            .match_v5()
            .get_match_ids_by_puuid(
                AMERICAS,
                &query.puuid,
                query.count.or(Some(MAX_MATCHES)),
                None,
                None,
                query.start_time,
                None,
                None,
            )
            .await
            .unwrap()
    }
}

impl Publish for MatchIdsRequester {
    type Input = GetMatchIdsQuery;
    type Output = Vec<String>;

    /// Push a [`GetMatchesQuery`] to the queue to fetch match IDs for.
    async fn push(&self, data: Self::Input) {
        let mut lock = self.matches_queue.lock().await;
        lock.push_back(data);
    }

    /// Loop the matches queue, fetching Match IDs for each `[GetMatchesQuery]`.
    ///
    /// Fetched data is pushed to the provided publishing channel.
    #[tracing::instrument]
    async fn start(&self, publishing_channel: UnboundedSender<Self::Output>) {
        loop {
            let mut lock = self.matches_queue.lock().await;
            if let Some(matches_query) = lock.pop_front() {
                drop(lock);
                let mut match_ids = self.get_matches(&matches_query).await;
                // Reverse the match IDs to iterate in the correct order
                match_ids.reverse();
                debug!("Got match IDs: {match_ids:?}");
                publishing_channel.send(match_ids).unwrap();
            }
        }
    }
}
