use serde::Deserialize;
use std::path::Path;
use tokio::fs::read_to_string;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub database_url: String,
    pub rgapi_key: String,
    pub iteration_secs: u64,
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
        config.rgapi_key = std::env::var("RGAPI_KEY").ok().unwrap_or(config.rgapi_key);
        config.iteration_secs = std::env::var("ITERATION_SECS")
            .map(|f| f.parse().expect("Should be numeric"))
            .ok()
            .unwrap_or(config.iteration_secs);

        Ok(config)
    }

    async fn load_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let contents = read_to_string(path).await?;
        Ok(toml::from_str(&contents)?)
    }
}
