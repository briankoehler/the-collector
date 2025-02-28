use chrono::TimeDelta;
use config::Config;
use handler::{account::AccountHandler, match_data::MatchDataHandler, match_ids::MatchIdsHandler};
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
use std::sync::Arc;
use the_collector_db::{DbHandler, SqlitePoolOptions};
use the_collector_ipc::{r#pub::IpcPublisher, IPC_SUMMONER_MATCH_PATH};
use tokio::sync::mpsc::unbounded_channel;
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod handler;
mod riot_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env();
    setup_tracing_subscriber();

    info!("Loading configuration");
    let config = Config::load(std::env::args().nth(1)).await?;

    // Setup Riot API
    info!("Setting up Riot API client");
    let riot_api = Arc::new(RiotApi::new(config.rgapi_key));

    // Setup DB Client
    info!("Setting up DB client");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
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

    let match_data_handler = MatchDataHandler::new(
        db_handler.clone(),
        match_rx,
        IpcPublisher::new(IPC_SUMMONER_MATCH_PATH)?,
    );
    tokio::task::spawn(match_data_handler.start());

    let match_ids_handler = MatchIdsHandler::new(db_handler.clone(), matches_rx, match_requester);
    tokio::task::spawn(match_ids_handler.start());

    info!("Starting main loop");
    loop {
        debug!("Sleeping {}s...", config.iteration_secs);
        tokio::time::sleep(std::time::Duration::from_secs(config.iteration_secs)).await;

        // Start with looping summoners, because that's what we're using to query
        // the API. If we started with guilds or followings, we might end up sending
        // duplicate requests (or have to implement logic to avoid duplicates)
        let summoners = db_handler.get_summoners().await?;
        for summoner in summoners {
            // If we have a latest match to use, use that to determine when to query from.
            // Otherwise, use the time of the summoner being added to the database. This
            // avoids the workaround used for a long time in which the last match prior to
            // the summoner being added to the database had to be added and processed first.
            let start_time = match db_handler
                .get_summoner_latest_match(&summoner.puuid)
                .await?
            {
                Some(latest_match) => latest_match
                    .start_time
                    .checked_add_signed(TimeDelta::seconds(latest_match.duration))
                    .expect("Time fits"),
                None => summoner.create_time,
            }
            .and_utc()
            .timestamp();

            let query = GetMatchIdsQuery {
                puuid: summoner.puuid,
                start_time: Some(start_time),
                count: None,
            };
            debug!("GetMatchIdsQuery: {query:?}");
            matches_requester.push(query).await;
        }
    }
}

fn load_env() {
    match dotenvy::dotenv() {
        Ok(path) => info!("Overriding config with values from {path:?}"),
        Err(e) if e.not_found() => info!("No env file found — only using values from config"),
        Err(e) => error!("Failed to load env file: {e:?}"),
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
