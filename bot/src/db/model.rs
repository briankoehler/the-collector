#![allow(unused)]
use sqlx::prelude::FromRow;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Debug, FromRow)]
pub struct Guild {
    pub id: i64,
    pub channel_id: Option<i64>,
}

#[derive(Debug, FromRow)]
pub struct Summoner {
    pub puuid: String,
    pub game_name: String,
    pub tag: String,
}

#[derive(Debug, FromRow)]
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

#[derive(Debug, FromRow)]
pub struct GuildFollowing {
    pub guild_id: i64,
    pub puuid: String,
}

#[derive(Debug, FromRow)]
pub struct SummonerMatch {
    pub puuid: String,
    pub match_id: String,
    pub kills: u8,
    pub deaths: u8,
    pub assists: u8,
    pub champion_id: u16,
    pub position: String,
    pub longest_time_living: u64,
    pub time_dead: u64,
    pub team_id: u16,
}
