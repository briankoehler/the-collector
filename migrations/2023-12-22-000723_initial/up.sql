CREATE TABLE account (
    puuid TEXT NOT NULL PRIMARY KEY,
    game_name TEXT NOT NULL,
    tag_line TEXT NOT NULL
);

CREATE TABLE match (
    id TEXT NOT NULL PRIMARY KEY,
    start_timestamp DATETIME NOT NULL,
    duration BigInt NOT NULL,
    queue_id BigInt NOT NULL,
    game_version TEXT NOT NULL,
    game_mode TEXT NOT NULL,
    winning_team_id BigInt NOT NULL,
    surrender BOOLEAN NOT NULL
);

CREATE TABLE match_stats(
    id TEXT NOT NULL PRIMARY KEY,
    puuid TEXT NOT NULL,
    match_id TEXT NOT NULL,
    kills BigInt NOT NULL,
    deaths BigInt NOT NULL,
    assists BigInt NOT NULL,
    champion_id BigInt NOT NULL,
    position TEXT NOT NULL,
    longest_time_spent_living BigInt NOT NULL,
    time_spent_dead BigInt NOT NULL,
    team_id BigInt NOT NULL,
    FOREIGN KEY(puuid) REFERENCES account(puuid),
    FOREIGN KEY(match_id) REFERENCES match(id)
);
