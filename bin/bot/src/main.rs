use async_trait::async_trait;
use command::Data;
use the_collector_db::{DbHandler, SqlitePoolOptions};
use poise::{
    serenity_prelude::{Client, Context, EventHandler, GatewayIntents, Ready},
    Framework, FrameworkOptions,
};
use riven::RiotApi;
use the_collector_ipc::{sub::IpcSubscriber, SummonerMatchQuery, IPC_SUMMONER_MATCH_PATH};
use std::sync::Arc;
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod command;

struct Handler {
    subscriber: Arc<IpcSubscriber<SummonerMatchQuery>>,
    db_handler: Arc<DbHandler>,
}

// TODO: Add event for adding guild to DB on join
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // TODO: Refactor/Clean-up
        // TODO: Investigate alternative methods of creating this task
        info!("{} has connected", ready.user.name);

        let subscriber = self.subscriber.clone();
        let db_handler = self.db_handler.clone();
        tokio::task::spawn(async move {
            loop {
                let summoner_match_query = subscriber.recv().await.unwrap();
                debug!("Got summoner match query: {summoner_match_query:?}");
                let Some(summoner_match) = db_handler.get_summoner_match(&summoner_match_query.puuid, &summoner_match_query.match_id).await.unwrap() else {
                    error!("Failed to get summoner match from {summoner_match_query:?}");
                    continue;
                };

                // TODO: Evaluate as int
                let is_int = true;

                if is_int {
                    debug!("Evaluated match as int: (PUUID: {:?}, Match ID: {:?})", summoner_match.puuid, summoner_match.match_id);
                    
                    // TODO: Construct message
                    let message = format!("{} just died {} times.", summoner_match.puuid, summoner_match.deaths);

                    let followers = db_handler.get_following_guilds(&summoner_match.puuid).await.unwrap();
                    debug!("Sending a message to {} guilds", followers.len());
                    for follower in followers {
                        let channel = ctx.http.get_channel((follower.channel_id.unwrap() as u64).into()).await.unwrap().guild().unwrap();
                        if let Err(e) = channel.say(&ctx.http, &message).await {
                            error!("Failed sending message: {e:?}");
                        }
                    }
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Get values from env file");
    setup_tracing_subscriber();

    info!("Setting up DB client");
    let db_uri = std::env::var("DATABASE_URL").expect("Failed to get DATABASE_URL");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_uri)
        .await
        .expect("Should connect to database");
    let db_handler = Arc::new(DbHandler::new(pool));

    let token = std::env::var("DISCORD_TOKEN").expect("Failed to get DISCORD_TOKEN");
    let intents = GatewayIntents::all();

    // Setup Riot API
    info!("Setting up Riot API client");
    let api_key = std::env::var("RGAPI_KEY").expect("Failed to get RGAPI_KEY");
    let riot_api = RiotApi::new(api_key);

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
                    riot_api,
                })
            })
        })
        .build();

    // TODO: Consolidate the event handler to the poise framework builder
    let subscriber = Arc::new(IpcSubscriber::new(IPC_SUMMONER_MATCH_PATH).unwrap());
    let mut client = Client::builder(token, intents)
        .framework(framework)
        .event_handler(Handler { subscriber, db_handler })
        .await
        .expect("Client should be created");

    info!("Starting client");
    client.start().await.expect("Client runs forever")
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
