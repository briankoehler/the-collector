// @generated automatically by Diesel CLI.

diesel::table! {
    account (puuid) {
        puuid -> Text,
        game_name -> Text,
        tag_line -> Text,
    }
}

diesel::table! {
    #[sql_name = "match"]
    match_ (id) {
        id -> Text,
        start_timestamp -> Timestamp,
        duration -> BigInt,
        queue_id -> BigInt,
        game_version -> Text,
        game_mode -> Text,
        winning_team_id -> BigInt,
        surrender -> Bool,
    }
}

diesel::table! {
    match_stats (id) {
        id -> Text,
        puuid -> Text,
        match_id -> Text,
        kills -> BigInt,
        deaths -> BigInt,
        assists -> BigInt,
        champion_id -> BigInt,
        position -> Text,
        longest_time_spent_living -> BigInt,
        time_spent_dead -> BigInt,
        team_id -> BigInt,
    }
}

diesel::joinable!(match_stats -> account (puuid));
diesel::joinable!(match_stats -> match_ (match_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    match_,
    match_stats,
);
