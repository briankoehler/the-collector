use serde::{Deserialize, Serialize};

pub mod r#pub;
pub mod sub;

pub const IPC_SUMMONER_MATCH_PATH: &str = "ipc:///tmp/int.ipc";

#[derive(Debug, Serialize, Deserialize)]
pub struct SummonerMatchQuery {
    pub puuid: String,
    pub match_id: String,
}
