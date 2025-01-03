use anyhow::Context;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Display};

const VERSIONS_URL: &str = "https://ddragon.leagueoflegends.com/api/versions.json";

#[derive(Debug, Deserialize)]
struct ChampionMap {
    pub data: HashMap<String, Champion>,
}

#[derive(Debug, Deserialize)]
struct Champion {
    key: String,
    name: String,
}

// Add other data dragon information as needed here
#[derive(Debug)]
pub struct DataDragonBlob {
    /// Map of champion ID to champion name
    pub champions: HashMap<u16, String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GameVersion(pub String);

impl GameVersion {
    pub async fn to_data_dragon_version(self) -> anyhow::Result<DataDragonVersion> {
        let version = self.0.split('.').take(2).collect::<Vec<&str>>().join(".");

        // TODO: Create a cache so that I/O can be avoided if possible
        let versions = fetch_versions().await?;
        for foo in versions {
            if foo.starts_with(&version) {
                return Ok(DataDragonVersion(foo));
            }
        }
        anyhow::bail!("Failed to find matching Data Dragon version");
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DataDragonVersion(String);

impl Display for DataDragonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug)]
pub struct DataDragon {
    cache: HashMap<DataDragonVersion, DataDragonBlob>,
}

impl DataDragon {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Retrieve a champion name given the champion ID and data dragon version. Will add
    /// the version's data to the cache if not yet present.
    pub async fn get_champion_name(
        &mut self,
        version: &DataDragonVersion,
        champion_id: u16,
    ) -> anyhow::Result<Option<String>> {
        match self.cache.get(version) {
            Some(blob) => Ok(blob.champions.get(&champion_id).cloned()),
            None => {
                let champions = fetch_champions(version).await?;
                let blob = self.insert_blob(version.clone(), champions)?;
                Ok(blob.champions.get(&champion_id).cloned())
            }
        }
    }

    fn insert_blob(
        &mut self,
        version: DataDragonVersion,
        champions: ChampionMap,
    ) -> anyhow::Result<&DataDragonBlob> {
        let mut champions_map = HashMap::new();
        for champion in champions.data.values() {
            champions_map.insert(champion.key.parse()?, champion.name.clone());
        }
        self.cache.insert(
            version.clone(),
            DataDragonBlob {
                champions: champions_map,
            },
        );
        self.cache
            .get(&version)
            .context("Failed to get Data Dragon blob from cache after insertion")
    }
}

async fn fetch_versions() -> anyhow::Result<Vec<String>> {
    Ok(reqwest::get(VERSIONS_URL).await?.json().await?)
}

async fn fetch_champions(version: &DataDragonVersion) -> anyhow::Result<ChampionMap> {
    let url = format!("https://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/champion.json");
    Ok(reqwest::get(url).await?.json::<ChampionMap>().await?)
}
