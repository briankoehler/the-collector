use riven::models::account_v1::Account;
use riven::models::match_v5::Match;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Error, Pool, Sqlite};

mod model;

// TODO: Job queue for DB tasks?

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
    pub async fn get_summoner(&self, puuid: &str) -> Result<model::Summoner, Error> {
        sqlx::query_as!(
            model::Summoner,
            "SELECT * FROM summoner WHERE puuid = ?",
            puuid
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Get all summoners from the database.
    pub async fn get_summoners(&self) -> Result<Vec<model::Summoner>, Error> {
        sqlx::query_as!(model::Summoner, "SELECT * FROM summoner")
            .fetch_all(&self.pool)
            .await
    }

    /// Get all guilds from the database.
    pub async fn get_guilds(&self) -> Result<Vec<model::Guild>, Error> {
        sqlx::query_as!(model::Guild, "SELECT * FROM guild")
            .fetch_all(&self.pool)
            .await
    }

    /// Get all guilds from the database.
    pub async fn get_matches(&self, match_ids: &[String]) -> Result<Vec<model::Match>, Error> {
        let match_ids = match_ids.join(", ");
        sqlx::query_as!(
            model::Match,
            "SELECT * FROM match WHERE id IN (?)",
            match_ids
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get guild followings that match the provided guild ID.
    pub async fn get_guild_follows(
        &self,
        guild_id: &str,
    ) -> Result<Vec<model::GuildFollowing>, Error> {
        sqlx::query_as!(
            model::GuildFollowing,
            "SELECT * FROM guild_following WHERE guild_id = ?",
            guild_id
        )
        .fetch_all(&self.pool)
        .await
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
    }

    /// Get the latest match information and respective stats of a PUUID.
    pub async fn get_following_guilds(&self, puuid: &str) -> Result<Vec<model::Guild>, Error> {
        sqlx::query_as!(model::Guild,
            "SELECT guild.* FROM guild_following INNER JOIN guild ON guild.id = guild_following.guild_id WHERE guild_following.puuid = ?", puuid
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Insert account data
    pub async fn insert_summoner(&self, account: Account) -> Result<SqliteQueryResult, Error> {
        sqlx::query("INSERT INTO summoner (puuid, game_name, tag) VALUES (?, ?, ?)")
            .bind(account.puuid)
            .bind(account.game_name.unwrap())
            .bind(account.tag_line.unwrap())
            .execute(&self.pool)
            .await
    }

    /// Insert match data
    pub async fn insert_match(&self, data: &Match) -> Result<SqliteQueryResult, Error> {
        let winning_team_id = get_winning_team(&data);
        let surrender = get_surrender(&data);

        sqlx::query("INSERT INTO match (id, start_time, duration, queue_id, game_version, game_mode, winning_team_id, surrender)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&data.metadata.match_id)
            .bind(chrono::DateTime::from_timestamp_millis(data.info.game_start_timestamp).unwrap())
            .bind(data.info.game_duration * 1000) // Convert to milliseconds to match start timestamp
            .bind(u16::from(data.info.queue_id))
            .bind(&data.info.game_version)
            .bind(&data.info.game_mode.to_string())
            .bind(winning_team_id)
            .bind(surrender)
            .execute(&self.pool)
            .await
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
            .unwrap();

        sqlx::query("INSERT INTO summoner_match (puuid, match_id, kills, deaths, assists, champion_id, position, longest_time_living, time_dead, team_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(puuid)
            .bind(&data.metadata.match_id)
            .bind(summoner_stats.kills)
            .bind(summoner_stats.deaths)
            .bind(summoner_stats.assists)
            .bind(i16::from(summoner_stats.champion().unwrap()))
            .bind(&summoner_stats.team_position)
            .bind(summoner_stats.longest_time_spent_living)
            .bind(summoner_stats.total_time_spent_dead)
            .bind(u16::from(summoner_stats.team_id))
            .execute(&self.pool)
            .await
    }
}

fn get_winning_team(data: &Match) -> u16 {
    data.info
        .participants
        .iter()
        .find(|participant| participant.win)
        .unwrap()
        .team_id
        .into()
}

fn get_surrender(data: &Match) -> bool {
    data.info
        .participants
        .first()
        .unwrap()
        .game_ended_in_surrender
}
