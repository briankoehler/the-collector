use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    ParseChampionError(#[from] riven::consts::ParseChampionError),
    #[error("missing data: {0} is not available")]
    MissingData(String),
    #[error("not enough matches for leaderboard")]
    NotEnoughLeaderboardMatches,
    #[error("time is out of range")]
    DateTimeOutOfRange,
}
