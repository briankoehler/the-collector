use crate::db::schema::account;
use crate::db::schema::match_;
use crate::db::schema::match_stats;
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection};
use riven::consts::RegionalRoute::AMERICAS;
use riven::models::match_v5::Match;
use riven::RiotApi;

// TODO: Move to a runtime configuration.
/// Seconds added to queries for new matches. This is necessary because
/// start_timestamp + duration of the most recent match does not filter it.
const START_TIME_BUFFER_SECS: i64 = 30;

// TODO: Support Non-Americas routes
/// Fetches new matches for the accounts found from the provided database
/// connection. Remains inactive until `start` is called.
pub struct MatchFetcher<'a> {
    accounts: Vec<String>,
    riot_api_key: String,
    conn: &'a mut SqliteConnection,
}

impl<'a> MatchFetcher<'a> {
    /// Creates a new `MatchFetcher`. Will panic if `RIOT_API_KEY` is not set or
    /// it fails to query the accounts table from the DB.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        let accounts = account::table.select(account::puuid).load(conn).unwrap();
        let riot_api_key = std::env::var("RIOT_API_KEY").unwrap();
        Self {
            accounts,
            riot_api_key,
            conn,
        }
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
            self.update_accounts();
            for puuid in self.accounts.clone() {
                println!("Starting PUUID: {}", puuid);
                // Get the latest match stats from DB for PUUID
                // If found, determine end time and get new matches and data from Riot API
                // If not found, get latest match and add to DB if necessary (and new match stats)
                match self.get_latest_local_match_id(&puuid) {
                    Ok(local_match_id) => {
                        println!("Found local match ID: {}", local_match_id);
                        let match_data = self.get_local_match_data(&local_match_id);
                        let calculated_start_time = match_data.start_timestamp.timestamp()
                            + match_data.duration
                            + START_TIME_BUFFER_SECS;

                        let new_matches = self
                            .get_api_matches(&puuid, Some(calculated_start_time))
                            .await;
                        println!("Got {} new matches from API", new_matches.len());
                        for new_match in new_matches {
                            let new_match_data = self.get_api_match_data(&new_match).await;
                            self.handle_api_match(new_match_data, puuid.as_str()).await;
                        }
                    }
                    Err(error) => {
                        println!("Query error");
                        if let diesel::result::Error::NotFound = error {
                            println!("No match found");
                            let new_match = self.get_api_matches(&puuid, None).await;
                            let new_match = new_match.first().unwrap();
                            let new_match_data = self.get_api_match_data(new_match).await;
                            self.handle_api_match(new_match_data, puuid.as_str()).await;
                        }
                    }
                }
            }
            println!("Sleeping 60 seconds...");
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }

    // TODO: Trigger this from updates to the DB?
    fn update_accounts(&mut self) {
        self.accounts = account::table
            .select(account::puuid)
            .load(self.conn)
            .unwrap();
    }

    /// Inserts missing Match and MatchStat from provided API match data.
    async fn handle_api_match(&mut self, new_match_data: Match, puuid: &str) {
        let new_cached_match = crate::db::model::Match::new(&new_match_data);
        let new_cached_match_stats = crate::db::model::MatchStat::new(puuid, &new_match_data);

        let inserted_match_count = new_cached_match
            .insert_into(match_::table)
            .on_conflict_do_nothing()
            .execute(self.conn)
            .unwrap();
        let inserted_match_stats_count = new_cached_match_stats
            .insert_into(match_stats::table)
            .execute(self.conn)
            .unwrap();

        println!("Inserted match count: {}", inserted_match_count);
        println!("Inserted match stats count: {}", inserted_match_stats_count);
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
    fn get_local_match_data(&mut self, match_id: &str) -> crate::db::model::Match {
        match_::table.find(match_id).first(self.conn).unwrap()
    }

    /// Gets match IDs from Riot API using the provided PUUID, and start time if provided.  
    /// Does not use other filters in query.
    async fn get_api_matches(&self, puuid: &String, start_time: Option<i64>) -> Vec<String> {
        RiotApi::new(self.riot_api_key.as_str())
            .match_v5()
            .get_match_ids_by_puuid(
                AMERICAS,
                puuid.as_str(),
                None,
                None,
                None,
                start_time,
                None,
                None,
            )
            .await
            .unwrap()
    }

    /// Get match data from Riot APII using the provided match ID.
    async fn get_api_match_data(&self, match_id: &str) -> Match {
        RiotApi::new(self.riot_api_key.as_str())
            .match_v5()
            .get_match(AMERICAS, match_id)
            .await
            .unwrap()
            .unwrap()
    }
}
