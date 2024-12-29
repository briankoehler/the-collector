use chrono::TimeDelta;
use db::DbHandler;
use handler::{account::AccountHandler, match_data::MatchDataHandler, match_ids::MatchIdsHandler};
use ipc::{BytesSender, INT_IPC_PATH};
use nng::Socket;
use riot_api::{
    account::AccountRequester,
    match_data::MatchDataRequester,
    match_ids::{GetMatchIdsQuery, MatchIdsRequester},
    Publish,
};
use riven::{
    models::{account_v1::Account, match_v5::Match},
    RiotApi,
};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

mod db;
mod evaluator;
mod handler;
mod ipc;
mod riot_api;

const ITERATION_SECS: u64 = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    setup_tracing_subscriber();

    // Setup Riot API
    info!("Setting up Riot API client");
    let api_key = std::env::var("RGAPI_KEY").expect("Failed to get RGAPI_KEY");
    let riot_api = Arc::new(RiotApi::new(api_key));

    // Setup DB Client
    info!("Setting up DB client");
    let db_uri = std::env::var("DATABASE_URL").expect("Failed to get DATABASE_URL");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_uri)
        .await?;
    let db_handler = Arc::new(DbHandler::new(pool));

    // Setup Riot API channels
    info!("Setting up channels");
    let (account_tx, account_rx) = unbounded_channel::<Account>();
    let (match_tx, match_rx) = unbounded_channel::<Match>();
    let (matches_tx, matches_rx) = unbounded_channel::<Vec<String>>();

    // Start API Queues
    info!("Starting Requester tasks");
    let account_requester = Arc::new(AccountRequester::new(riot_api.clone()));
    tokio::task::spawn({
        let account_requester = account_requester.clone();
        async move {
            account_requester.start(account_tx).await;
        }
    });
    let match_requester = Arc::new(MatchDataRequester::new(riot_api.clone()));
    tokio::task::spawn({
        let match_requester = match_requester.clone();
        async move {
            match_requester.start(match_tx).await;
        }
    });
    let matches_requester = Arc::new(MatchIdsRequester::new(riot_api.clone()));
    tokio::task::spawn({
        let matches_requester = matches_requester.clone();
        async move {
            matches_requester.start(matches_tx).await;
        }
    });

    info!("Starting Handler tasks");
    let account_handler = AccountHandler::new(db_handler.clone(), account_rx);
    tokio::task::spawn(account_handler.start());

    let client = Arc::new(Socket::new(nng::Protocol::Push0)?);
    client.dial(INT_IPC_PATH)?;
    let ipc_sender = BytesSender::new(client);
    let match_data_handler = MatchDataHandler::new(db_handler.clone(), match_rx, ipc_sender);
    tokio::task::spawn(match_data_handler.start());

    let match_ids_handler = MatchIdsHandler::new(db_handler.clone(), matches_rx, match_requester);
    tokio::task::spawn(match_ids_handler.start());

    info!("Starting main loop");
    loop {
        debug!("Sleeping {ITERATION_SECS}s...");
        tokio::time::sleep(std::time::Duration::from_secs(ITERATION_SECS)).await;

        // Start with looping summoners, because that's what we're using to query
        // the API. If we started with guilds or followings, we might end up sending
        // duplicate requests (or have to implement logic to avoid duplicates)
        let summoners = db_handler.get_summoners().await?;
        for summoner in summoners {
            let latest_match = db_handler
                .get_summoner_latest_match(&summoner.puuid)
                .await?;
            // If we have data already, do not provide a count to fetch. Otherwise, only
            // get the most recent match. Additionally, calculate the start time to query
            // by based on the end time of the last match
            let count = match latest_match {
                Some(_) => None,
                None => Some(1),
            };
            let start_time = latest_match.map(|latest_match| {
                latest_match
                    .start_time
                    .checked_add_signed(TimeDelta::milliseconds(latest_match.duration))
                    .expect("Time fits")
                    .and_utc()
                    .timestamp()
            });

            let query = GetMatchIdsQuery {
                puuid: summoner.puuid.into(),
                start_time,
                count,
            };
            info!("GetMatchIdsQuery: {query:?}");
            matches_requester.push(query).await;
        }
    }
}

fn setup_tracing_subscriber() {
    let layer = fmt::layer()
        .pretty()
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(false)
        .with_target(false);
    tracing_subscriber::registry()
        .with(layer)
        .with(EnvFilter::from_default_env())
        .init();
}
