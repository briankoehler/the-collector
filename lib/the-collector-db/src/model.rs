use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::JsonValue;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Guild {
    pub id: i64,
    pub channel_id: Option<i64>,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct Summoner {
    pub puuid: String,
    pub game_name: String,
    pub tag: String,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct Match {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub duration: PgInterval,
    pub queue_id: i64,
    pub game_version: String,
    pub game_mode: String,
    pub winning_team_id: i64,
    pub surrender: bool,
    pub raw: Option<JsonValue>,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GuildFollowing {
    pub guild_id: i64,
    pub puuid: String,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct SummonerMatch {
    pub puuid: String,
    pub match_id: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub champion_id: i64,
    pub position: Option<String>,
    pub longest_time_living: PgInterval,
    pub time_dead: PgInterval,
    pub team_id: i64,
    pub raw: Option<JsonValue>,
    pub create_time: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct SummonerAggregateStats {
    pub game_name: String,
    pub tag: String,
    pub num_matches: Option<i64>,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub total_duration: PgInterval,
    pub total_time_dead: PgInterval,
}
