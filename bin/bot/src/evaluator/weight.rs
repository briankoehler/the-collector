use serde::Deserialize;
use the_collector_db::model::SummonerMatch;

#[derive(Debug, PartialEq, PartialOrd, Deserialize, Default)]
pub struct WeightedKda(pub f32);

#[derive(Debug, PartialEq, PartialOrd, Deserialize)]
pub struct Weights {
    kill_weight: f32,
    death_weight: f32,
    assist_weight: f32,
}

impl Weights {
    pub fn calculate_weighted_kda(&self, stats: &SummonerMatch) -> WeightedKda {
        let inner = (self.kill_weight * stats.kills as f32)
            + (self.death_weight * stats.deaths as f32)
            + (self.assist_weight * stats.assists as f32);
        WeightedKda(inner)
    }
}
