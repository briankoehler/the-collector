use crate::evaluator::MatchStatsEvaluator;
use crate::message::MessageBuilder;
use async_trait::async_trait;
use poise::serenity_prelude::{
    Context, EventHandler, Guild, GuildChannel, Message, Ready, UnavailableGuild,
};
use std::sync::Arc;
use the_collector_db::DbHandler;
use the_collector_ipc::sub::IpcSubscriber;
use the_collector_ipc::SummonerMatchQuery;
use tracing::{debug, error, info};

pub struct Handler {
    pub subscriber: Arc<IpcSubscriber<SummonerMatchQuery>>,
    pub db_handler: Arc<DbHandler>,
    pub evaluator: Arc<MatchStatsEvaluator>,
    pub message_builder: Arc<MessageBuilder>,
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

                    let summoner = db_handler
                        .get_summoner(&summoner_match.puuid)
                        .await
                        .unwrap()
                        .unwrap();
                    let message = message_builder.build_message(&summoner_match, &summoner);

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

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        // TODO: Use cached is_new after learning about it
        info!("Adding guild with ID {:?} to database.", guild.id);
        self.db_handler.insert_guild(guild.id.into()).await.unwrap();
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        incomplete: UnavailableGuild,
        _full: Option<Guild>,
    ) {
        if !incomplete.unavailable {
            info!("Removing guild with ID {:?} from database.", incomplete.id);
            self.db_handler
                .delete_guild(incomplete.id.into())
                .await
                .unwrap();
        }
    }

    async fn channel_delete(
        &self,
        _ctx: Context,
        channel: GuildChannel,
        _messages: Option<Vec<Message>>,
    ) {
        let result = self
            .db_handler
            .delete_channel(channel.id.into())
            .await
            .unwrap();
        if result.rows_affected() >= 1 {
            info!(
                "Deleted {} channel IDs from database.",
                result.rows_affected()
            );
        }
    }
}
