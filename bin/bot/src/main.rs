use anyhow::Context as _;
use async_trait::async_trait;
use command::Data;
use evaluator::MatchStatsEvaluator;
use message::MessageBuilder;
use poise::{
    serenity_prelude::{
        Client, Connection, Context, EventHandler, GatewayIntents, Guild, GuildChannel, Message,
        Ready, UnavailableGuild,
    },
    Framework, FrameworkOptions,
};
use riven::RiotApi;
use std::sync::Arc;
use the_collector_db::{DbHandler, SqlitePoolOptions};
use the_collector_ipc::{sub::IpcSubscriber, SummonerMatchQuery, IPC_SUMMONER_MATCH_PATH};
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod command;
mod evaluator;
mod message;

struct Handler {
    subscriber: Arc<IpcSubscriber<SummonerMatchQuery>>,
    db_handler: Arc<DbHandler>,
    evaluator: Arc<MatchStatsEvaluator>,
    message_builder: Arc<MessageBuilder>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // TODO: Refactor/Clean-up
        // TODO: Investigate alternative methods of creating this task
        info!("{} has connected", ready.user.name);

        let subscriber = self.subscriber.clone();
        let db_handler = self.db_handler.clone();
        let evaluator = self.evaluator.clone();
        let message_builder = self.message_builder.clone();
        tokio::task::spawn(async move {
            loop {
                let summoner_match_query = subscriber.recv().await.unwrap();
                debug!("Got summoner match query: {summoner_match_query:?}");
                let Some(summoner_match) = db_handler
                    .get_summoner_match(&summoner_match_query.puuid, &summoner_match_query.match_id)
                    .await
                    .unwrap()
                else {
                    error!("Failed to get summoner match from {summoner_match_query:?}");
                    continue;
                };

                if evaluator.is_int(&summoner_match) {
                    debug!(
                        "Evaluated match as int: (PUUID: {:?}, Match ID: {:?})",
                        summoner_match.puuid, summoner_match.match_id
                    );

                    let message = message_builder.build_message(&summoner_match);

                    let followers = db_handler
                        .get_following_guilds(&summoner_match.puuid)
                        .await
                        .unwrap();
                    debug!("Sending a message to {} guilds", followers.len());
                    for follower in followers {
                        let channel = ctx
                            .http
                            .get_channel((follower.channel_id.unwrap() as u64).into())
                            .await
                            .unwrap()
                            .guild()
                            .unwrap();
                        if let Err(e) = channel.say(&ctx.http, &message).await {
                            error!("Failed sending message: {e:?}");
                        }
                    }
                }
            }
        });
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        todo!("If guild is new, add to database (without a channel ID")
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, full: Option<Guild>) {
        todo!("Remove guild from database (or add a new column to set to non-active)")
    }

    async fn channel_delete(
        &self,
        ctx: Context,
        channel: GuildChannel,
        messages: Option<Vec<Message>>,
    ) {
        todo!("Update database if deleted channel is the notification channel")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    setup_tracing_subscriber();

    info!("Setting up DB client");
    let db_uri = std::env::var("DATABASE_URL")?;
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_uri)
        .await
        .context("Failed to connect to database")?;
    let db_handler = Arc::new(DbHandler::new(pool));

    let token = std::env::var("DISCORD_TOKEN")?;
    let intents = GatewayIntents::all();

    // Setup Riot API
    info!("Setting up Riot API client");
    let api_key = std::env::var("RGAPI_KEY")?;
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
    let mut client = Client::builder(token, intents)
        .framework(framework)
        .event_handler(Handler {
            db_handler,
            subscriber: Arc::new(IpcSubscriber::new(IPC_SUMMONER_MATCH_PATH)?),
            evaluator: Arc::new(MatchStatsEvaluator::new()),
            message_builder: Arc::new(MessageBuilder::new()),
        })
        .await
        .context("Failed to create client")?;

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
