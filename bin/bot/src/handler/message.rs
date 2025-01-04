use crate::{evaluator::MatchStatsEvaluator, message::MessageBuilder};
use anyhow::Context as _;
use poise::serenity_prelude::Http;
use std::sync::Arc;
use the_collector_db::DbHandler;
use the_collector_ipc::{sub::IpcSubscriber, SummonerMatchQuery};
use tracing::{debug, error};

#[derive(Debug)]
pub struct MessageHandler {
    pub db_handler: Arc<DbHandler>,
    pub subscriber: IpcSubscriber<SummonerMatchQuery>,
    pub evaluator: MatchStatsEvaluator,
    pub message_builder: MessageBuilder,
    pub http: Arc<Http>,
}

impl MessageHandler {
    pub async fn start(&self) {
        loop {
            if let Err(e) = self.run().await {
                error!("Error handling summoner match: {e:?}");
            }
        }
    }

    async fn run(&self) -> anyhow::Result<()> {
        let summoner_match_query = self.subscriber.recv().await?;
        debug!("Got summoner match query: {summoner_match_query:?}");

        let summoner_match = self
            .db_handler
            .get_summoner_match(&summoner_match_query.puuid, &summoner_match_query.match_id)
            .await?
            .context("Failed to get summoner match from {summoner_match_query:?}")?;
        let match_data = self
            .db_handler
            .get_match(&summoner_match.match_id)
            .await?
            .context("Failed to get corresponding match")?;

        if !self.evaluator.is_int(&summoner_match, &match_data) {
            return Ok(());
        }
        debug!(
            "Evaluated match as int: (PUUID: {:?}, Match ID: {:?})",
            summoner_match.puuid, summoner_match.match_id
        );

        let summoner = self
            .db_handler
            .get_summoner(&summoner_match.puuid)
            .await?
            .context("No summoner with PUUID found in database")?;
        let message = self
            .message_builder
            .build_message(&summoner_match, &summoner);
        let followers = self
            .db_handler
            .get_following_guilds(&summoner_match.puuid)
            .await?;

        debug!("Sending a message to {} guilds", followers.len());
        for follower in followers {
            let Some(channel_id) = follower.channel_id else {
                debug!("Skipping {:?} because no channel ID set yet", follower.id);
                continue;
            };

            let channel = self
                .http
                .get_channel((channel_id as u64).into())
                .await?
                .guild()
                .context("Found non-guild channel ID in database")?;
            if let Err(e) = channel.say(&self.http, &message).await {
                error!("Failed sending message: {e:?}");
            }
        }

        Ok(())
    }
}
