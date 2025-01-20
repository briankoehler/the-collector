use rand::seq::SliceRandom;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use the_collector_db::model;
use the_collector_evaluation::label::IntLevel;

#[derive(Debug, Deserialize)]
pub struct MessageBuilder {
    templates: HashMap<IntLevel, Vec<String>>,
}

impl MessageBuilder {
    pub async fn new(templates_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let contents = tokio::fs::read_to_string(templates_path).await?;
        Ok(toml::from_str(&contents)?)
    }

    // TODO: Add more here
    pub fn build_message(
        &self,
        summoner_match: &model::SummonerMatch,
        summoner: &model::Summoner,
        level: &IntLevel,
    ) -> String {
        let templates = self
            .templates
            .get(level)
            .expect("Templates for given level");

        let message = templates
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
