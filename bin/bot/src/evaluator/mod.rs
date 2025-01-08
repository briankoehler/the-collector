use serde::Deserialize;
use std::collections::HashMap;
use the_collector_db::model::{Match, SummonerMatch};
use weight::{WeightedKda, Weights};

pub mod weight;

#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Top,
    Jungle,
    Mid,
    Bot,
    Support,
    Other,
}

impl From<&str> for Role {
    fn from(value: &str) -> Self {
        // Match to values from Riot API
        match value.to_lowercase().as_str() {
            "top" => Role::Top,
            "jungle" => Role::Jungle,
            "middle" => Role::Mid,
            "bottom" => Role::Bot,
            "utility" => Role::Support,
            _ => Role::Other,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct MatchStatsEvaluator {
    kda_weights: HashMap<Role, Weights>,
    kda_limit: WeightedKda,
}

impl MatchStatsEvaluator {
    pub fn is_int(&self, match_stats: &SummonerMatch, _match_data: &Match) -> bool {
        // Don't send a message for insignificant scoreline
        if match_stats.deaths <= 4 && match_stats.kills >= 1 {
            return false;
        }

        // If their weighted KDA is less than <= 0, evaluate as an int
        let role = match_stats
            .position
            .as_ref()
            .map(String::as_str)
            .unwrap_or("other")
            .into();
        self.kda_weights[&role].calculate_weighted_kda(match_stats) <= self.kda_limit
    }
}
