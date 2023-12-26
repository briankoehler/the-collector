use crate::db::establish_connection;
use crate::db::model;
use crate::db::schema::match_stats;
use crate::server::routes::{jsonify, JsonString};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

pub async fn get_match_stats() -> JsonString {
    let mut conn = establish_connection();

    let matches: Vec<model::MatchStat> = match_stats::table.load(&mut conn).unwrap();
    jsonify(&matches)
}

pub async fn get_match_stat(Path(id): Path<String>) -> Result<JsonString, StatusCode> {
    let mut conn = establish_connection();

    match match_stats::table
        .find(id)
        .first::<model::MatchStat>(&mut conn)
    {
        Ok(found_match) => Ok(jsonify(&found_match)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn post_match_stats(Json(payload): Json<model::MatchStat>) -> JsonString {
    let mut conn = establish_connection();

    let result = diesel::insert_into(match_stats::table)
        .values(&payload)
        .returning(model::MatchStat::as_returning())
        .get_result(&mut conn)
        .expect("Error inserting new account");
    jsonify(&result)
}

pub async fn put_match_stats(Json(payload): Json<model::MatchStat>) -> JsonString {
    let mut conn = establish_connection();

    let num_updated = diesel::update(match_stats::table)
        .set(&payload)
        .execute(&mut conn)
        .unwrap();
    jsonify(num_updated)
}

pub async fn delete_match_stats(Path(puuid): Path<String>) -> JsonString {
    let mut conn = establish_connection();
    let deleted: Vec<model::MatchStat> = diesel::delete(match_stats::table.find(puuid))
        .get_results(&mut conn)
        .unwrap();
    jsonify(deleted)
}
