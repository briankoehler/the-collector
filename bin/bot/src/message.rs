use rand::seq::SliceRandom;
use std::path::Path;
use the_collector_db::model;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug)]
pub struct MessageBuilder {
    templates: Vec<String>,
}

impl MessageBuilder {
    pub async fn new(templates_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(templates_path).await?;
        let reader = BufReader::new(file);

        let mut stream = reader.lines();
        let mut templates = Vec::new();
        while let Some(line) = stream.next_line().await? {
            templates.push(line);
        }

        Ok(Self { templates })
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
