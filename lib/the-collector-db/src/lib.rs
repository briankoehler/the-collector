use error::Error;
use riven::models::account_v1::Account;
use riven::models::match_v5::Match;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};

// Re-export so that clients can avoid having sqlx as a dependency
pub use sqlx::sqlite::SqlitePoolOptions;

pub mod error;
pub mod model;

/// Draft, Ranked Solo, Ranked Flex
const QUEUE_IDS: [u16; 3] = [400, 420, 440];

// TODO: Job queue for DB tasks?
// TODO: Improve this organization

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

    /// Get a summoner from the database given the PUUID.
    pub async fn get_summoner(&self, puuid: &str) -> Result<Option<model::Summoner>, Error> {
        sqlx::query_as!(
            model::Summoner,
            "SELECT * FROM summoner WHERE puuid = ?",
            puuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get all summoners from the database.
    pub async fn get_summoners(&self) -> Result<Vec<model::Summoner>, Error> {
        sqlx::query_as!(model::Summoner, "SELECT * FROM summoner")
            .fetch_all(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Get all guilds from the database.
    pub async fn get_guilds(&self) -> Result<Vec<model::Guild>, Error> {
        sqlx::query_as!(model::Guild, "SELECT * FROM guild")
            .fetch_all(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Get all matches from the database.
    pub async fn get_matches(&self, match_ids: &[String]) -> Result<Vec<model::Match>, Error> {
        let queue_parameters = match_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ");
        let raw_query = format!("SELECT * FROM match WHERE id IN ({queue_parameters})");
        let mut query = sqlx::query_as(&raw_query);
        for match_id in match_ids {
            query = query.bind(match_id);
        }
        query.fetch_all(&self.pool).await.map_err(Error::SqlxError)
    }

    /// Get all matches from the database.
    pub async fn get_match(&self, match_id: &str) -> Result<Option<model::Match>, Error> {
        sqlx::query_as!(model::Match, "SELECT * FROM match WHERE id = ?", match_id)
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
            "SELECT * FROM guild_following WHERE guild_id = ?",
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
            "SELECT match.* FROM match INNER JOIN summoner_match ON match.id = summoner_match.match_id
            WHERE summoner_match.puuid = ? ORDER BY start_time DESC LIMIT 1", puuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Get the latest match information and respective stats of a PUUID.
    pub async fn get_following_guilds(&self, puuid: &str) -> Result<Vec<model::Guild>, Error> {
        sqlx::query_as!(model::Guild,
            "SELECT guild.* FROM guild_following INNER JOIN guild ON guild.id = guild_following.guild_id WHERE guild_following.puuid = ?", puuid
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::SqlxError)
    }

    /// Insert account data - does not return an error if an account with the same PUUID
    /// already exists (keeping the original).
    pub async fn insert_summoner(&self, account: &Account) -> Result<SqliteQueryResult, Error> {
        let now = Utc::now().naive_utc();
        let game_name = account
            .game_name
            .as_ref()
            .ok_or(Error::MissingData("Game Name".into()))?;
        let tag = &account
            .tag_line
            .as_ref()
            .ok_or(Error::MissingData("Tag".into()))?;
        sqlx::query!(
            "INSERT OR IGNORE
            INTO summoner (puuid, game_name, tag, create_time)
            VALUES (?, ?, ?, ?)",
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
    pub async fn insert_guild(&self, guild_id: u64) -> Result<SqliteQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!("INSERT OR IGNORE INTO guild (id) VALUES (?)", guild_id)
            .execute(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Delete guild data
    pub async fn delete_guild(&self, guild_id: u64) -> Result<SqliteQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!("DELETE FROM guild WHERE id = ?", guild_id)
            .execute(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    /// Insert match data
    pub async fn insert_match(&self, data: &Match) -> Result<SqliteQueryResult, Error> {
        let winning_team_id = get_winning_team(data)?;
        let surrender = get_surrender(data)?;

        sqlx::query("INSERT INTO match (id, start_time, duration, queue_id, game_version, game_mode, winning_team_id, surrender)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&data.metadata.match_id)
            .bind(DateTime::from_timestamp_millis(data.info.game_start_timestamp).ok_or(Error::DateTimeOutOfRange)?)
            .bind(data.info.game_duration)
            .bind(u16::from(data.info.queue_id))
            .bind(&data.info.game_version)
            .bind(data.info.game_mode.to_string())
            .bind(winning_team_id)
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
    ) -> Result<SqliteQueryResult, Error> {
        let summoner_stats = data
            .info
            .participants
            .iter()
            .find(|p| p.puuid == puuid)
            .ok_or(Error::MissingData("Matching PUUID".into()))?;

        sqlx::query("INSERT INTO summoner_match (puuid, match_id, kills, deaths, assists, champion_id, position, longest_time_living, time_dead, team_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(puuid)
            .bind(&data.metadata.match_id)
            .bind(summoner_stats.kills)
            .bind(summoner_stats.deaths)
            .bind(summoner_stats.assists)
            .bind(i16::from(summoner_stats.champion()?))
            .bind(&summoner_stats.team_position)
            .bind(summoner_stats.longest_time_spent_living)
            .bind(summoner_stats.total_time_spent_dead)
            .bind(u16::from(summoner_stats.team_id))
            .execute(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    pub async fn delete_channel(&self, channel_id: u64) -> Result<SqliteQueryResult, Error> {
        let channel_id = channel_id as i64;
        sqlx::query!(
            "UPDATE guild SET channel_id = NULL WHERE channel_id = ?",
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
    ) -> Result<SqliteQueryResult, Error> {
        let guild_id = guild_id as i64;
        let channel_id = channel_id.map(|id| id as i64);
        sqlx::query!(
            "UPDATE guild SET channel_id = ? WHERE id = ?",
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
        let queue_parameters = QUEUE_IDS.map(|_| "?").join(", ");
        let raw_query = format!(
            "SELECT summoner_match.* FROM summoner_match
            INNER JOIN guild_following ON guild_following.puuid = summoner_match.puuid
            INNER JOIN match ON summoner_match.match_id = match.id
            WHERE guild_following.guild_id = ? AND match.queue_id IN ({})
            ORDER BY deaths DESC LIMIT ?",
            queue_parameters
        );
        let mut query = sqlx::query_as(&raw_query).bind(guild_id as i64);
        for queue_id in QUEUE_IDS {
            query = query.bind(queue_id);
        }
        query
            .bind(size as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::SqlxError)
    }

    pub async fn insert_guild_following(
        &self,
        guild_id: u64,
        puuid: &str,
    ) -> Result<SqliteQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!(
            "INSERT INTO guild_following (guild_id, puuid) VALUES (?, ?)",
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
            "SELECT * FROM summoner_match WHERE puuid = ? AND match_id = ?",
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
    ) -> Result<SqliteQueryResult, Error> {
        let guild_id = guild_id as i64;
        sqlx::query!(
            "DELETE FROM guild_following WHERE guild_id = ? AND puuid = ?",
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
            "SELECT * FROM summoner WHERE game_name = ? AND tag = ?",
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
                game_name AS "game_name!", tag AS "tag!", COUNT(*) AS num_matches,
                SUM(kills) AS "kills!: u16", SUM(deaths) AS "deaths!", SUM(assists) AS "assists!",
                SUM(duration) AS "total_duration!", SUM(time_dead) AS "total_time_dead!"
            FROM summoner_match 
            INNER JOIN summoner ON summoner_match.puuid = summoner.puuid
            INNER JOIN match ON summoner_match.match_id = match.id
            WHERE game_name = ? and tag = ?
            HAVING num_matches > 0
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
