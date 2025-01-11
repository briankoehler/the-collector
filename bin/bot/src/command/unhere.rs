use crate::command::{CommandError, Data};
use anyhow::Context;

/// Unset the channel that notifications are sent to
#[poise::command(slash_command, guild_only)]
pub async fn unhere(ctx: poise::Context<'_, Data, CommandError>) -> Result<(), CommandError> {
    let guild_id = ctx.guild_id().context("Trying to get guild ID")?;

    ctx.data()
        .db_handler
        .update_channel(guild_id.into(), None)
        .await?;

    ctx.say("Unset the notification channel").await?;
    Ok(())
}
