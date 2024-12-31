use rand::seq::SliceRandom;
use the_collector_db::model;

// TODO: Load from configuration file
const TEMPLATES: [&str; 4] = [
    "%s just died **%d times**.",
    "Solid **%d bomb** by %s.",
    "**BREAKING NEWS:** %S INTS ANOTHER GAME WITH **%d DEATHS**.",
    "**NEWS FLASH:** %S DROPS A **%d DEATH** GAME."
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
    pub fn build_message(&self, summoner_match: &model::SummonerMatch) -> String {
        let message = self
            .templates
            .choose(&mut rand::thread_rng())
            .expect("Templates is unexpectedly empty");
        let message = message.replace("%s", &summoner_match.puuid);
        let message = message.replace("%S", &summoner_match.puuid.to_uppercase());
        message.replace("%d", &summoner_match.deaths.to_string())
    }
}
