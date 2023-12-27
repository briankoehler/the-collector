use crate::db::schema::account;
use crate::db::{establish_connection, model};
use crate::server::routes::{jsonify, JsonString};
use axum::http::StatusCode;
use axum::{extract::Path, Json};
use diesel::dsl::sql;
use diesel::query_builder::{AsQuery, DebugQuery};
use diesel::sqlite::Sqlite;
use diesel::{debug_query, Insertable, IntoSql, QueryDsl, RunQueryDsl, SelectableHelper};
use riven::RiotApi;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewAccountData {
    game_name: String,
    tag_line: String,
}

pub async fn post_account_by_name(Json(payload): Json<NewAccountData>) -> JsonString {
    let mut conn = establish_connection();
    let api_key = std::env::var("RIOT_API_KEY").expect("DATABASE_URL must be set");
    let riot_api = RiotApi::new(api_key);

    let account_data = riot_api
        .account_v1()
        .get_by_riot_id(
            riven::consts::RegionalRoute::AMERICAS,
            &payload.game_name,
            &payload.tag_line,
        )
        .await
        .unwrap()
        .unwrap();

    let new_account = model::Account::new(&account_data).unwrap();

    let result = diesel::insert_into(account::table)
        .values(&new_account)
        .get_result::<model::Account>(&mut conn)
        .expect("Error inserting new account");
    jsonify(&result)
}

pub async fn get_accounts() -> JsonString {
    let mut conn = establish_connection();

    let accounts: Vec<model::Account> = account::table.load(&mut conn).unwrap();
    jsonify(&accounts)
}

pub async fn get_account(Path(puuid): Path<String>) -> Result<JsonString, StatusCode> {
    let mut conn = establish_connection();

    match account::table
        .find(puuid)
        .first::<model::Account>(&mut conn)
    {
        Ok(found_account) => Ok(jsonify(&found_account)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn put_account(Json(payload): Json<model::Account>) -> JsonString {
    let mut conn = establish_connection();

    let num_updated = diesel::update(account::table)
        .set(&payload)
        .execute(&mut conn)
        .unwrap();
    jsonify(num_updated)
}

pub async fn delete_account(Path(puuid): Path<String>) -> JsonString {
    let mut conn = establish_connection();
    let deleted: Vec<model::Account> = diesel::delete(account::table.find(puuid))
        .get_results(&mut conn)
        .unwrap();
    jsonify(deleted)
}
