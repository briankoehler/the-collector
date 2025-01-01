use std::collections::HashMap;
use the_collector_db::model::SummonerMatch;

// TODO: Move to a configuration file
// TODO: Research values to use here
lazy_static::lazy_static! {
    static ref THRESHOLDS: HashMap<Role, Threshold> = {
        HashMap::from([
            (Role::Top, Threshold { kda: 1.5, deaths: 9 }),
            (Role::Jungle, Threshold { kda: 1.5, deaths: 9 }),
            (Role::Mid, Threshold { kda: 1.5, deaths: 9 }),
            (Role::Bot, Threshold { kda: 1.5, deaths: 9 }),
            (Role::Support, Threshold { kda: 1.5, deaths: 9 }),
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
    kda: f32,
    deaths: u8,
}

impl Threshold {
    // TODO: Improve this logic
    fn is_int(&self, stats: &SummonerMatch) -> bool {
        if stats.deaths >= self.deaths.into() {
            return true;
        }
        let kda = (stats.kills as f32 + stats.assists as f32) / stats.deaths as f32;
        if kda <= self.kda {
            return true;
        }
        return false;
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

    pub fn is_int(&self, match_stats: &SummonerMatch) -> bool {
        // TODO: Get threshold to use from position
        self.thresholds[&Role::Top].is_int(match_stats)
    }
}
