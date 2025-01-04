use chrono::TimeDelta;
use std::collections::HashMap;
use the_collector_db::model::{Match, SummonerMatch};

// TODO: Move to a configuration file
// TODO: Research values to use here
lazy_static::lazy_static! {
    static ref THRESHOLDS: HashMap<Role, Threshold> = {
        HashMap::from([
            (Role::Top, Threshold { weighted_kda: WeightedKda(1.5), minutes_per_death: MinutesPerDeath(3.5) }),
            (Role::Jungle, Threshold { weighted_kda: WeightedKda(1.5), minutes_per_death: MinutesPerDeath(3.5) }),
            (Role::Mid, Threshold { weighted_kda: WeightedKda(1.5), minutes_per_death: MinutesPerDeath(3.5) }),
            (Role::Bot, Threshold { weighted_kda: WeightedKda(1.5), minutes_per_death: MinutesPerDeath(3.5) }),
            (Role::Support, Threshold { weighted_kda: WeightedKda(1.5), minutes_per_death: MinutesPerDeath(3.5) }),
            (Role::Other, Threshold { weighted_kda: WeightedKda(1.5), minutes_per_death: MinutesPerDeath(3.5) }),
        ])
    };
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Role {
    Top,
    Jungle,
    Mid,
    Bot,
    Support,
    Other,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct WeightedKda(f32);

impl From<&SummonerMatch> for WeightedKda {
    fn from(stats: &SummonerMatch) -> Self {
        let inner = (stats.kills as f32 + (stats.assists as f32 * 0.5)) / stats.deaths as f32;
        Self(inner)
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct MinutesPerDeath(f32);

impl From<(&SummonerMatch, &Match)> for MinutesPerDeath {
    fn from(data: (&SummonerMatch, &Match)) -> Self {
        let minutes = TimeDelta::milliseconds(data.1.duration).num_minutes();
        let inner = minutes as f32 / data.0.deaths as f32;
        Self(inner)
    }
}

/// A threshold of what is considered an int and what is not. The values
/// in a threshold are inclusive to something being evaluated positively
/// as an int.
///
/// For example, if the threshold has a deaths value of 10 and the stat
/// block being tested has a deaths value of 10, then it is evaluated
/// positively as an int.
#[derive(Debug)]
struct Threshold {
    weighted_kda: WeightedKda,
    minutes_per_death: MinutesPerDeath,
}

impl Threshold {
    fn is_int(&self, stats: &SummonerMatch, match_data: &Match) -> bool {
        if self.weighted_kda >= stats.into() {
            return true;
        }
        if self.minutes_per_death >= (stats, match_data).into() {
            return true;
        }
        false
    }
}

#[derive(Debug)]
pub struct MatchStatsEvaluator {
    thresholds: &'static HashMap<Role, Threshold>,
}

impl MatchStatsEvaluator {
    pub fn new() -> Self {
        Self {
            thresholds: &THRESHOLDS,
        }
    }

    pub fn is_int(&self, match_stats: &SummonerMatch, match_data: &Match) -> bool {
        match match_stats
            .position
            .as_ref()
            .unwrap_or(&String::from("Other"))
        {
            val if val == "TOP" => &self.thresholds[&Role::Top],
            val if val == "JUNGLE" => &self.thresholds[&Role::Jungle],
            val if val == "MIDDLE" => &self.thresholds[&Role::Mid],
            val if val == "BOTTOM" => &self.thresholds[&Role::Bot],
            val if val == "UTILITY" => &self.thresholds[&Role::Support],
            _ => &self.thresholds[&Role::Other],
        }
        .is_int(match_stats, match_data)
    }
}
