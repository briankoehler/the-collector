use riven::consts::RegionalRoute::AMERICAS;
use riven::models::match_v5::Match;
use riven::RiotApi;

pub struct RivenWrapper<'a> {
    riot_api_key: &'a str,
}

impl<'a> RivenWrapper<'a> {
    pub fn new(riot_api_key: &'a str) -> Self {
        Self { riot_api_key }
    }

    /// Gets match IDs from Riot API using the provided PUUID, and start time if provided.  
    /// Does not use other filters in query. If an error is encountered, returns an empty vector.
    pub async fn get_api_matches(&self, puuid: &String, start_time: Option<i64>) -> Vec<String> {
        RiotApi::new(self.riot_api_key)
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
            .unwrap_or_else(|e| {
                println!("Failed to get matches: {e}");
                vec![]
            })
    }

    /// Get match data from Riot APII using the provided match ID.
    pub async fn get_api_match_data(&self, match_id: &str) -> Option<Match> {
        RiotApi::new(self.riot_api_key)
            .match_v5()
            .get_match(AMERICAS, match_id)
            .await
            .unwrap_or_else(|e| {
                println!("Failed to get match data: {e}");
                None
            })
    }
}
