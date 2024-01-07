use chrono::NaiveDateTime;
use diesel::{Connection, SqliteConnection};
use riven::models::{account_v1::Account, match_v5::Match};

pub mod model;
pub mod schema;

/// Gets a connection to the Sqlite database found at the
/// URL of the `DATABASE_URL` environment variable
pub fn establish_connection() -> SqliteConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).unwrap()
}

impl model::Account {
    /// Creates a new Account model from a Riot API account response.
    pub fn new(riven_account: &Account) -> Result<Self, String> {
        if let (Some(game_name), Some(tag_line)) = (
            riven_account.game_name.clone(),
            riven_account.tag_line.clone(),
        ) {
            Ok(Self {
                puuid: riven_account.puuid.clone(),
                game_name,
                tag_line,
            })
        } else {
            Err("Foo".into())
        }
    }
}

impl model::Match {
    /// Creates a new Match model from a Riot API match response.
    pub fn new(riven_match: &Match) -> Self {
        let mut participants = riven_match.info.participants.iter();
        let winning_team_id = participants
            .find(|p| p.win)
            .expect("No winning team found")
            .team_id as i64;
        // TODO: Fix this for arena games
        // let surrender = participants
        //     .find(|p| !p.win)
        //     .expect("No loser found")
        //     .game_ended_in_surrender;
        let surrender = false;

        Self {
            id: riven_match.metadata.match_id.clone(),
            start_timestamp: NaiveDateTime::from_timestamp_millis(
                riven_match.info.game_start_timestamp,
            )
            .unwrap(),
            duration: riven_match.info.game_duration,
            queue_id: riven_match.info.queue_id.0 as i64,
            game_version: riven_match.info.game_version.clone(),
            game_mode: riven_match.info.game_mode.to_string(),
            winning_team_id,
            surrender,
        }
    }
}

impl model::MatchStat {
    /// Creates a new MatchStat model from a Riot API match response and
    /// provided PUUID.
    pub fn new(puuid: &str, riven_match: &Match) -> Self {
        let participant = riven_match
            .info
            .participants
            .iter()
            .find(|p| p.puuid == puuid)
            .unwrap();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            puuid: puuid.into(),
            match_id: riven_match.metadata.match_id.clone(),
            kills: participant.kills.into(),
            deaths: participant.deaths.into(),
            assists: participant.assists.into(),
            champion_id: participant.champion().unwrap().0.into(),
            position: participant.team_position.clone(),
            longest_time_spent_living: participant.longest_time_spent_living.into(),
            time_spent_dead: participant.total_time_spent_dead.into(),
            team_id: (participant.team_id as u16).into(),
        }
    }
}
