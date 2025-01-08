use anyhow::Context as _;
use command::Data;
use config::Config;
use ddragon::DataDragon;
use handler::bot::BotHandler;
use handler::message::MessageHandler;
use message::MessageBuilder;
use poise::serenity_prelude::{Client, GatewayIntents};
use poise::{Framework, FrameworkOptions};
use riven::RiotApi;
use std::sync::Arc;
use the_collector_db::{DbHandler, SqlitePoolOptions};
use the_collector_ipc::{sub::IpcSubscriber, IPC_SUMMONER_MATCH_PATH};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod command;
mod config;
mod ddragon;
mod evaluator;
mod handler;
mod message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    setup_tracing_subscriber();

    info!("Loading configuration");
    let config = Config::load(std::env::args().nth(1)).await?;

    info!("Setting up DB client");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;
    let db_handler = Arc::new(DbHandler::new(pool));

    // Setup Riot API
    info!("Setting up Riot API client");
    let riot_api = RiotApi::new(config.rgapi_key);

    let db_handler_clone = db_handler.clone();
    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                command::about(),
                command::follow(),
                command::here(),
                command::leaderboard(),
                command::list(),
                command::stats(),
                command::unfollow(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    db_handler: db_handler_clone,
                    data_dragon: Mutex::new(DataDragon::new()),
                    riot_api,
                })
            })
        })
        .build();

    // TODO: Consolidate the event handler to the poise framework builder
    let mut client = Client::builder(config.discord_token, GatewayIntents::all())
        .framework(framework)
        .event_handler(BotHandler {
            db_handler: db_handler.clone(),
        })
        .await
        .context("Failed to create client")?;

    let summoner_match_handler = MessageHandler {
        db_handler: db_handler.clone(),
        subscriber: IpcSubscriber::new(IPC_SUMMONER_MATCH_PATH)?,
        evaluator: config.match_stats_evaluator,
        message_builder: MessageBuilder::new(config.message_templates_path).await?,
        http: client.http.clone(),
    };
    info!("Starting Summoner Match Handler");
    tokio::task::spawn(async move { summoner_match_handler.start().await });

    info!("Starting client");
    client.start().await.context("Client exited its loop")?;

    Ok(())
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
