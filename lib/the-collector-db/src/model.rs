use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono::NaiveDateTime};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Guild {
    pub id: i64,
    pub channel_id: Option<i64>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Summoner {
    pub puuid: String,
    pub game_name: String,
    pub tag: String,
    pub create_time: NaiveDateTime,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub start_time: NaiveDateTime,
    pub duration: i64,
    pub queue_id: i64,
    pub game_version: String,
    pub game_mode: String,
    pub winning_team_id: i64,
    pub surrender: bool,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GuildFollowing {
    pub guild_id: i64,
    pub puuid: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SummonerMatch {
    pub puuid: String,
    pub match_id: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub champion_id: i64,
    pub position: Option<String>,
    pub longest_time_living: i64,
    pub time_dead: i64,
    pub team_id: i64,
}
