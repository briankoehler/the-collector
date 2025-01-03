use async_trait::async_trait;
use poise::serenity_prelude::{
    Context, EventHandler, Guild, GuildChannel, Message, Ready, UnavailableGuild,
};
use std::sync::Arc;
use the_collector_db::DbHandler;
use tracing::{error, info};

#[derive(Debug)]
pub struct BotHandler {
    pub db_handler: Arc<DbHandler>,
}

#[async_trait]
impl EventHandler for BotHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} has connected", ready.user.name);
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        // TODO: Use cached is_new after learning about it
        info!("Adding guild with ID {:?} to database.", guild.id);
        if let Err(e) = self.db_handler.insert_guild(guild.id.into()).await {
            error!(
                "Failed to add guild with ID {:?} to database: {e:?}",
                guild.id
            );
        }
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        incomplete: UnavailableGuild,
        _full: Option<Guild>,
    ) {
        if !incomplete.unavailable {
            info!("Removing guild with ID {:?} from database.", incomplete.id);
            if let Err(e) = self.db_handler.delete_guild(incomplete.id.into()).await {
                error!(
                    "Failed to delete guild with ID {:?} from database: {e:?}",
                    incomplete.id
                );
            }
        }
    }

    async fn channel_delete(
        &self,
        _ctx: Context,
        channel: GuildChannel,
        _messages: Option<Vec<Message>>,
    ) {
        match self.db_handler.delete_channel(channel.id.into()).await {
            Ok(result) => {
                if result.rows_affected() >= 1 {
                    info!(
                        "Deleted {} channel IDs from database.",
                        result.rows_affected()
                    );
                }
            }
            Err(e) => error!(
                "Failed trying to delete references to channel {channel:?} from database: {e:?}"
            ),
        }
    }
}
