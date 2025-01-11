use crate::command::{CommandError, Data};
use anyhow::Context;

/// Set the channel that notifications are sent to
#[poise::command(slash_command, guild_only)]
pub async fn here(ctx: poise::Context<'_, Data, CommandError>) -> Result<(), CommandError> {
    let guild_id = ctx.guild_id().context("Trying to get guild ID")?;

    let channel_id = ctx.channel_id();
    ctx.data()
        .db_handler
        .update_channel(guild_id.into(), Some(channel_id.into()))
        .await?;

    let message = format!(
        "Setting notification channel to **#{}**.",
        ctx.guild_channel()
            .await
            .context("Trying to get guild channel")?
            .name
    );
    ctx.reply(message).await?;
    Ok(())
}
