use error::Error;
use riven::models::account_v1::Account;
use riven::models::match_v5::Match;
use sqlx::postgres::PgQueryResult;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

// Re-export so that clients can avoid having sqlx as a dependency
pub use sqlx::postgres::types;
pub use sqlx::postgres::PgPoolOptions;

pub mod error;
pub mod model;

/// Draft, Ranked Solo, Ranked Flex
const QUEUE_IDS: [i16; 3] = [400, 420, 440];

// TODO: Job queue for DB tasks?
// TODO: Improve this organization

/// Wrapper around common database operations — by using this wrapper, clients
/// of [`DbHandler`] can remain database agnostic.
#[derive(Debug)]
pub struct DbHandler {
    pool: Pool<Postgres>,
}

impl DbHandler {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get a summoner from the database given the PUUID.
    pub async fn get_summoner(&self, puuid: &str) -> Result<Option<model::Summoner>, Error> {
        sqlx::query_as!(
            model::Summoner,
            "SELECT * FROM main.summoner WHERE puuid = $1",
            puuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get all summoners from the database.
    pub async fn get_summoners(&self) -> Result<Vec<model::Summoner>, Error> {
        sqlx::query_as!(model::Summoner, "SELECT * FROM main.summoner")
            .fetch_all(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Get all guilds from the database.
    pub async fn get_guilds(&self) -> Result<Vec<model::Guild>, Error> {
        sqlx::query_as!(model::Guild, "SELECT * FROM main.guild")
            .fetch_all(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Get all matches from the database.
    pub async fn get_matches(&self, match_ids: &[String]) -> Result<Vec<model::Match>, Error> {
        sqlx::query_as!(
            model::Match,
            r#"
            SELECT * FROM main.match
            WHERE id = ANY($1)
            "#,
            match_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get all matches from the database.
    pub async fn get_match(&self, match_id: &str) -> Result<Option<model::Match>, Error> {
        sqlx::query_as!(
            model::Match,
            "SELECT * FROM main.match WHERE id = $1",
            match_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get guild followings that match the provided guild ID.
    pub async fn get_guild_follows(
        &self,
        guild_id: u64,
    ) -> Result<Vec<model::GuildFollowing>, Error> {
        let guild_id = guild_id as i64;
        sqlx::query_as!(
            model::GuildFollowing,
            "SELECT * FROM main.guild_following WHERE guild_id = $1",
            guild_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get the latest match information and respective stats of a PUUID.
    pub async fn get_summoner_latest_match(
        &self,
        puuid: &str,
    ) -> Result<Option<model::Match>, Error> {
        sqlx::query_as!(model::Match,
            "SELECT match.* FROM main.match INNER JOIN main.summoner_match ON main.match.id = main.summoner_match.match_id
            WHERE main.summoner_match.puuid = $1 ORDER BY start_time DESC LIMIT 1", puuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get the latest match information and respective stats of a PUUID.
    pub async fn get_following_guilds(&self, puuid: &str) -> Result<Vec<model::Guild>, Error> {
        sqlx::query_as!(model::Guild,
            "SELECT guild.* FROM main.guild_following INNER JOIN main.guild ON main.guild.id = main.guild_following.guild_id WHERE main.guild_following.puuid = $1", puuid
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Insert account data - does not return an error if an account with the same PUUID
    /// already exists (keeping the original).
    pub async fn insert_summoner(&self, account: &Account) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        let game_name = account
            .game_name
            .as_ref()
            .ok_or(Error::MissingData("Game Name".into()))?;
        let tag = &account
            .tag_line
            .as_ref()
            .ok_or(Error::MissingData("Tag".into()))?;
        sqlx::query!(
            "INSERT INTO main.summoner (puuid, game_name, tag, create_time)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING",
            account.puuid,
            game_name,
            tag,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Insert guild data
    pub async fn insert_guild(&self, guild_id: u64) -> Result<PgQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!(
            "INSERT INTO main.guild (id) VALUES ($1) ON CONFLICT DO NOTHING",
            guild_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Delete guild data
    pub async fn delete_guild(&self, guild_id: u64) -> Result<PgQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!("DELETE FROM main.guild WHERE id = $1", guild_id)
            .execute(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Insert match data
    pub async fn insert_match(&self, data: &Match) -> Result<PgQueryResult, Error> {
        let winning_team_id = get_winning_team(data)?;
        let surrender = get_surrender(data)?;

        sqlx::query("INSERT INTO main.match (id, start_time, duration, queue_id, game_version, game_mode, winning_team_id, surrender)
            VALUES ($1, $2, CAST($3 * INTERVAL '1 second' AS interval), $4, $5, $6, $7, $8)")
            .bind(&data.metadata.match_id)
            .bind(DateTime::from_timestamp_millis(data.info.game_start_timestamp).ok_or(Error::DateTimeOutOfRange)?)
            .bind(data.info.game_duration)
            .bind(i32::from(u16::from(data.info.queue_id)))
            .bind(&data.info.game_version)
            .bind(data.info.game_mode.to_string())
            .bind(i32::from(winning_team_id))
            .bind(surrender)
            .execute(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Insert summoner match data
    pub async fn insert_summoner_match(
        &self,
        puuid: &str,
        data: &Match,
    ) -> Result<PgQueryResult, Error> {
        let summoner_stats = data
            .info
            .participants
            .iter()
            .find(|p| p.puuid == puuid)
            .ok_or(Error::MissingData("Matching PUUID".into()))?;

        sqlx::query("INSERT INTO main.summoner_match (puuid, match_id, kills, deaths, assists, champion_id, position, longest_time_living, time_dead, team_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, CAST($8 * INTERVAL '1 second' AS interval), CAST($9 * INTERVAL '1 second' AS interval), $10)")
            .bind(puuid)
            .bind(&data.metadata.match_id)
            .bind(summoner_stats.kills)
            .bind(summoner_stats.deaths)
            .bind(summoner_stats.assists)
            .bind(i16::from(summoner_stats.champion()?))
            .bind(&summoner_stats.team_position)
            .bind(summoner_stats.longest_time_spent_living)
            .bind(summoner_stats.total_time_spent_dead)
            .bind(i32::from(u16::from(summoner_stats.team_id)))
            .execute(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    pub async fn delete_channel(&self, channel_id: u64) -> Result<PgQueryResult, Error> {
        let channel_id = channel_id as i64;
        sqlx::query!(
            "UPDATE main.guild SET channel_id = NULL WHERE channel_id = $1",
            channel_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    pub async fn update_channel(
        &self,
        guild_id: u64,
        channel_id: Option<u64>,
    ) -> Result<PgQueryResult, Error> {
        let guild_id = guild_id as i64;
        let channel_id = channel_id.map(|id| id as i64);
        sqlx::query!(
            "UPDATE main.guild SET channel_id = $1 WHERE id = $2",
            channel_id,
            guild_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    pub async fn get_leaderboard(
        &self,
        guild_id: u64,
        size: usize,
    ) -> Result<Vec<model::SummonerMatch>, Error> {
        // TODO: Only get ints?
        let query = sqlx::query_as!(
            model::SummonerMatch,
            "SELECT main.summoner_match.* FROM main.summoner_match
            INNER JOIN main.guild_following ON main.guild_following.puuid = main.summoner_match.puuid
            INNER JOIN main.match ON main.summoner_match.match_id = main.match.id
            WHERE main.guild_following.guild_id = $1 AND main.match.queue_id = ANY($2)
            ORDER BY deaths DESC LIMIT $3",
            guild_id as i64,
            &QUEUE_IDS,
            size as i64,
        );
        query.fetch_all(&self.pool).await.map_err(Error::SqlxError)
    }

    pub async fn insert_guild_following(
        &self,
        guild_id: u64,
        puuid: &str,
    ) -> Result<PgQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!(
            "INSERT INTO main.guild_following (guild_id, puuid) VALUES ($1, $2)",
            guild_id,
            puuid
        )
        .execute(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    pub async fn get_summoner_match(
        &self,
        puuid: &str,
        match_id: &str,
    ) -> Result<Option<model::SummonerMatch>, Error> {
        sqlx::query_as!(
            model::SummonerMatch,
            "SELECT * FROM main.summoner_match WHERE puuid = $1 AND match_id = $2",
            puuid,
            match_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    pub async fn delete_guild_following(
        &self,
        guild_id: u64,
        puuid: &str,
    ) -> Result<PgQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!(
            "DELETE FROM main.guild_following WHERE guild_id = $1 AND puuid = $2",
            guild_id,
            puuid
        )
        .execute(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    pub async fn get_summoner_by_name(
        &self,
        name: &str,
        tag: &str,
    ) -> Result<Option<model::Summoner>, Error> {
        sqlx::query_as!(
            model::Summoner,
            "SELECT * FROM main.summoner WHERE game_name = $1 AND tag = $2",
            name,
            tag
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    pub async fn get_summoner_stats(
        &self,
        name: &str,
        tag: &str,
    ) -> Result<Option<model::SummonerAggregateStats>, Error> {
        sqlx::query_as!(
            model::SummonerAggregateStats,
            r#"SELECT
                game_name AS "game_name!", tag AS "tag!", COUNT(*) as "num_matches!",
                SUM(kills) AS "kills!", SUM(deaths) AS "deaths!", SUM(assists) AS "assists!",
                SUM(duration) AS "total_duration!", SUM(time_dead) AS "total_time_dead!"
            FROM main.summoner_match 
            INNER JOIN main.summoner ON main.summoner_match.puuid = main.summoner.puuid
            INNER JOIN main.match ON main.summoner_match.match_id = main.match.id
            WHERE game_name = $1 and tag = $2
            GROUP BY game_name, tag
            LIMIT 1"#,
            name,
            tag
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }
}

fn get_winning_team(data: &Match) -> Result<u16, Error> {
    Ok(data
        .info
        .participants
        .iter()
        .find(|participant| participant.win)
        .ok_or(Error::MissingData("Winner".into()))?
        .team_id
        .into())
}

fn get_surrender(data: &Match) -> Result<bool, Error> {
    Ok(data
        .info
        .participants
        .first()
        .ok_or(Error::MissingData("Participants".into()))?
        .game_ended_in_surrender)
}
