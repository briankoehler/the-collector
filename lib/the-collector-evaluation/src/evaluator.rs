use crate::label::IntLevel;
use crate::role::Role;
use crate::weight::{WeightedKda, Weights};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Range;
use the_collector_db::model::{Match, SummonerMatch};

#[derive(Debug)]
#[non_exhaustive]
pub struct Evaluation<'a> {
    pub level: &'a IntLevel,
}

#[derive(Debug, Deserialize, Default)]
pub struct MatchEvaluator {
    kda_weights: HashMap<Role, Weights>,
    kda_threshold: WeightedKda,
    level_ranges: HashMap<IntLevel, Range<u8>>,
}

impl MatchEvaluator {
    pub fn evaluate(&self, match_stats: &SummonerMatch, _match_data: &Match) -> Evaluation {
        let role = match_stats.position.as_deref().unwrap_or_default().into();
        let kda = self.kda_weights[&role].calculate_weighted_kda(match_stats);

        if kda > self.kda_threshold {
            return Evaluation {
                level: &IntLevel::Not,
            };
        };

        for (level, range) in &self.level_ranges {
            if range.contains(&(match_stats.deaths as u8)) {
                return Evaluation { level };
            }
        }

        // TODO: Skipping labeling for now, but would be cool to have

        Evaluation {
            level: &IntLevel::Not,
        }
    }
}
