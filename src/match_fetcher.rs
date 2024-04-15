use crate::db::schema::account;
use crate::db::schema::match_;
use crate::db::schema::match_stats;
use crate::match_subscriber::MatchSubscribe;
use crate::riven_wrapper::RivenWrapper;
use anyhow::anyhow;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection};
use riven::models::match_v5::Match;

// TODO: Move to a runtime configuration.
/// Seconds added to queries for new matches. This is necessary because
/// start_timestamp + duration of the most recent match does not filter it.
const START_TIME_BUFFER_SECS: i64 = 30;

// TODO: Support Non-Americas routes
/// Fetches new matches for the accounts found from the provided database
/// connection. Remains inactive until `start` is called.
pub struct MatchFetcher<'a> {
    riven: RivenWrapper<'a>,
    conn: &'a mut SqliteConnection,
    accounts: Vec<String>,
    match_subscribers: Vec<Box<&'a mut dyn MatchSubscribe>>,
}

impl<'a> MatchFetcher<'a> {
    /// Creates a new `MatchFetcher`. Will panic if it fails to query the
    /// accounts table from the DB.
    pub fn new(riot_api_key: &'a str, conn: &'a mut SqliteConnection) -> Self {
        let accounts = account::table
            .select(account::puuid)
            .load(conn)
            .unwrap_or_else(|e| {
                println!("Failed to get accounts: {e}");
                println!("Defaulting to an empty vector");
                vec![]
            });

        Self {
            accounts,
            riven: RivenWrapper::new(riot_api_key),
            conn,
            match_subscribers: vec![],
        }
    }

    pub fn add_match_subscriber(&mut self, subscriber: &'a mut impl MatchSubscribe) {
        self.match_subscribers.push(Box::new(subscriber));
    }

    /// Starts the `MatchFetcher`. This is an involved process, but can be broken down to:
    /// 1. Iterate each account PUUID
    /// 2. Get the latest match ID for the PUUID
    /// 3. If found, then get matches since then and add entries to the DB
    /// 4. If not found, then get the latest match and add entries to the DB
    /// 5. Sleep and then repeat
    pub async fn start(mut self) {
        loop {
            println!("Starting iteration");
            match self.get_accounts() {
                Ok(accounts) => self.accounts = accounts,
                Err(e) => println!("Failed to get new accounts: {e}"),
            }
            for puuid in self.accounts.clone() {
                println!("Starting PUUID: {puuid}");
                // Get the latest match stats from DB for PUUID
                // If found, determine end time and get new matches and data from Riot API
                // If not found, get latest match and add to DB if necessary (and new match stats)
                match self.get_latest_local_match_id(&puuid) {
                    Ok(local_match_id) => {
                        println!("Found local match ID: {local_match_id}");
                        let Some(match_data) = self.get_local_match_data(&local_match_id) else {
                            continue;
                        };
                        let calculated_start_time =
                            match_data.start_timestamp.and_utc().timestamp()
                                + match_data.duration
                                + START_TIME_BUFFER_SECS;

                        let new_matches = self
                            .riven
                            .get_api_matches(&puuid, Some(calculated_start_time))
                            .await;
                        println!("Got {} new matches from API", new_matches.len());
                        for new_match in new_matches {
                            let Some(new_match_data) =
                                self.riven.get_api_match_data(&new_match).await
                            else {
                                continue;
                            };
                            self.handle_api_match(new_match_data, puuid.as_str()).await;
                        }
                    }
                    Err(error) => {
                        println!("Query error");
                        if let diesel::result::Error::NotFound = error {
                            println!("No match found");
                            let new_match = self.riven.get_api_matches(&puuid, None).await;
                            let new_match = new_match.first().unwrap();
                            let Some(new_match_data) =
                                self.riven.get_api_match_data(new_match).await
                            else {
                                continue;
                            };
                            self.handle_api_match(new_match_data, puuid.as_str()).await;
                        }
                    }
                }
            }
            println!("Sleeping 60 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }

    // TODO: Trigger this from updates to the DB? Then other actors could add
    // new accounts
    fn get_accounts(&mut self) -> anyhow::Result<Vec<String>> {
        account::table
            .select(account::puuid)
            .load(self.conn)
            .map_err(|e| anyhow!(e))
    }

    /// Calls subscribers' match data handlers
    async fn handle_api_match(&mut self, new_match_data: Match, puuid: &str) {
        let new_cached_match = crate::db::model::Match::new(&new_match_data);
        let new_cached_match_stats = crate::db::model::MatchStat::new(puuid, &new_match_data);

        for subscriber in &mut self.match_subscribers {
            subscriber.handle_match(&new_cached_match).await;
            subscriber.handle_match_stats(&new_cached_match_stats).await;
        }
    }

    /// Gets the latest match ID that a PUUID was tied to from the local DB.
    fn get_latest_local_match_id(&mut self, puuid: &String) -> QueryResult<String> {
        match_stats::table
            .filter(match_stats::puuid.eq(puuid))
            .select(match_stats::match_id)
            .inner_join(match_::table)
            .order(match_::start_timestamp.desc())
            .first(self.conn)
    }

    /// Gets match information from local DB using the provided match ID.
    fn get_local_match_data(&mut self, match_id: &str) -> Option<crate::db::model::Match> {
        match_::table.find(match_id).first(self.conn).ok()
    }
}
