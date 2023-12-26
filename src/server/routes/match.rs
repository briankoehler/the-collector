use crate::db::establish_connection;
use crate::db::model;
use crate::db::schema::match_;
use crate::server::routes::{jsonify, JsonString};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};

pub async fn get_matches() -> JsonString {
    let mut conn = establish_connection();

    let matches: Vec<model::Match> = match_::table.load(&mut conn).unwrap();
    jsonify(&matches)
}

pub async fn get_match(Path(id): Path<String>) -> Result<JsonString, StatusCode> {
    let mut conn = establish_connection();

    match match_::table.find(id).first::<model::Match>(&mut conn) {
        Ok(found_match) => Ok(jsonify(&found_match)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn post_match(Json(payload): Json<model::Match>) -> JsonString {
    let mut conn = establish_connection();

    let result = diesel::insert_into(match_::table)
        .values(&payload)
        .returning(model::Match::as_returning())
        .get_result(&mut conn)
        .expect("Error inserting new account");
    jsonify(&result)
}

pub async fn put_match(Json(payload): Json<model::Match>) -> JsonString {
    let mut conn = establish_connection();

    let num_updated = diesel::update(match_::table)
        .set(&payload)
        .execute(&mut conn)
        .unwrap();
    jsonify(num_updated)
}

pub async fn delete_match(Path(puuid): Path<String>) -> JsonString {
    let mut conn = establish_connection();
    let deleted: Vec<model::Match> = diesel::delete(match_::table.find(puuid))
        .get_results(&mut conn)
        .unwrap();
    jsonify(deleted)
}
