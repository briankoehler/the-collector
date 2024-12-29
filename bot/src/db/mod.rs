use riven::models::account_v1::Account;
use sqlx::{sqlite::SqliteQueryResult, Error, Pool, Sqlite};

mod model;

/// Wrapper around common database operations — by using this wrapper, clients
/// of [`DbHandler`] can remain database agnostic.
#[derive(Debug)]
pub struct DbHandler {
    // TODO: Support Postgres
    pool: Pool<Sqlite>,
}

impl DbHandler {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn delete_channel_id(&self, guild_id: u64) -> Result<SqliteQueryResult, Error> {
        todo!()
    }

    pub async fn update_channel(
        &self,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<SqliteQueryResult, Error> {
        // TODO: Handle channel already existing
        sqlx::query("INSERT INTO guild (id, channel_id) VALUES (?, ?)")
            .bind(guild_id as i64)
            .bind(channel_id as i64)
            .execute(&self.pool)
            .await
    }

    /// Get guild followings that match the provided guild ID.
    pub async fn get_guild_follows(
        &self,
        guild_id: u64,
    ) -> Result<Vec<model::GuildFollowing>, Error> {
        let guild_id = guild_id as i64;
        sqlx::query_as!(
            model::GuildFollowing,
            "SELECT * FROM guild_following WHERE guild_id = ?",
            guild_id
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_leaderboard<const SIZE: usize>(
        &self,
        guild_id: u64,
    ) -> Result<[model::SummonerMatch; SIZE], Error> {
        todo!()
    }

    /// Insert account data
    pub async fn insert_summoner(&self, account: Account) -> Result<SqliteQueryResult, Error> {
        // TODO: Handle puuid already existing, i.e. name changes
        sqlx::query("INSERT INTO summoner (puuid, game_name, tag) VALUES (?, ?, ?)")
            .bind(account.puuid)
            .bind(account.game_name.unwrap())
            .bind(account.tag_line.unwrap())
            .execute(&self.pool)
            .await
    }

    pub async fn insert_guild_following(
        &self,
        guild_id: u64,
        puuid: &str,
    ) -> Result<SqliteQueryResult, Error> {
        sqlx::query("INSERT INTO guild_following (guild_id, puuid) VALUES (?, ?)")
            .bind(guild_id as i64)
            .bind(puuid)
            .execute(&self.pool)
            .await
    }
}
