use rand::seq::SliceRandom;
use the_collector_db::model;

// TODO: Load from configuration file
const TEMPLATES: [&str; 10] = [
    "%s just died **%d times**. Wow!",
    "Solid **%d bomb** by %s.",
    "**%d death** game coming from %s. Nice.",
    "Just a little bit of limit testing by %s, resulting in **%d deaths.**",
    "Mr. Inty Pants %s just inted **%d times!**",
    "Yikes, **%d deaths** for %s that last match.",
    "What a game by %s! **%k kills and %d deaths!**",
    "**BREAKING NEWS:** %S INTS ANOTHER GAME WITH **%d DEATHS**.",
    "**NEWS FLASH:** %S DROPS A **%d DEATH** GAME.",
    "Holy moly - **%d DEATHS** BY %S!!",
];

pub struct MessageBuilder {
    templates: Vec<String>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            templates: TEMPLATES.map(|s| s.to_string()).to_vec(),
        }
    }

    // TODO: Add more here
    pub fn build_message(
        &self,
        summoner_match: &model::SummonerMatch,
        summoner: &model::Summoner,
    ) -> String {
        let message = self
            .templates
            .choose(&mut rand::thread_rng())
            .expect("Templates is unexpectedly empty");
        message
            .replace("%s", &summoner.game_name)
            .replace("%S", &summoner.game_name.to_uppercase())
            .replace("%d", &summoner_match.deaths.to_string())
            .replace("%d", &summoner_match.deaths.to_string())
            .replace("%k", &summoner_match.kills.to_string())
    }
}
