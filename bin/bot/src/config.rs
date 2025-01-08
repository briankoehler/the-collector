use crate::evaluator::MatchStatsEvaluator;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs::read_to_string;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub database_url: String,
    pub discord_token: String,
    pub rgapi_key: String,
    pub message_templates_path: PathBuf,
    // TODO: Consider making this also a path
    pub match_stats_evaluator: MatchStatsEvaluator,
}

impl Config {
    pub async fn load(path: Option<impl AsRef<Path>>) -> anyhow::Result<Self> {
        let mut config = match path {
            Some(path) => Self::load_file(path).await?,
            None => Default::default(),
        };

        config.database_url = std::env::var("DATABASE_URL")
            .ok()
            .unwrap_or(config.database_url);
        config.discord_token = std::env::var("DISCORD_TOKEN")
            .ok()
            .unwrap_or(config.discord_token);
        config.rgapi_key = std::env::var("RGAPI_KEY").ok().unwrap_or(config.rgapi_key);
        config.message_templates_path = std::env::var("MESSAGE_TEMPLATES_PATH")
            .ok()
            .map(PathBuf::from)
            .unwrap_or(config.message_templates_path);

        Ok(config)
    }

    async fn load_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let contents = read_to_string(path).await?;
        Ok(toml::from_str(&contents)?)
    }
}
