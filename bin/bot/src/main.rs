use async_trait::async_trait;
use command::Data;
use int_bot_db::DbHandler;
use ipc::{BytesReceiver, IntNotification, INT_IPC_PATH};
use nng::{Protocol, Socket};
use poise::{
    serenity_prelude::{Client, Context, EventHandler, GatewayIntents, Ready},
    Framework, FrameworkOptions,
};
use riven::RiotApi;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod command;
mod ipc;

struct Handler {
    rx_int: Arc<Mutex<BytesReceiver<IntNotification>>>,
}

// TODO: Add event for adding guild to DB on join
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // TODO: Refactor/Clean-up
        // TODO: Investigate alternative methods of creating this task
        info!("{} has connected", ready.user.name);

        let rx_int = self.rx_int.clone();
        tokio::task::spawn(async move {
            loop {
                // let (mut stream, _) = rx_int.accept().await.unwrap();

                loop {
                    // stream.readable().await.unwrap();
                    // let mut buf = vec![];
                    // let num_bytes = stream.read_to_end(&mut buf).await.expect("No I/O errors");
                    // debug!("Read {num_bytes} bytes");

                    // let notification = bincode::deserialize::<IntNotification>(&buf).unwrap();

                    let mut lock = rx_int.lock().await;
                    let notification = lock.recv().await.unwrap();
                    info!("Got notification: {notification:?}");
                    let channel = ctx
                        .http
                        .get_channel(notification.channel_id.into())
                        .await
                        .unwrap()
                        .guild()
                        .unwrap();

                    if let Err(e) = channel.say(&ctx.http, notification.message).await {
                        error!("Failed sending message: {e:?}");
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
    let client = Socket::new(Protocol::Pull0).unwrap();
    client.listen(INT_IPC_PATH).unwrap();
    let rx_int = Arc::new(Mutex::new(BytesReceiver::new(Arc::new(client))));
    let mut client = Client::builder(token, intents)
        .framework(framework)
        .event_handler(Handler { rx_int })
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
