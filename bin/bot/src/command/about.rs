use crate::command::{CommandError, Data};

/// Display information about the bot (e.g. version)
#[poise::command(slash_command, guild_only)]
pub async fn about(ctx: poise::Context<'_, Data, CommandError>) -> Result<(), CommandError> {
    let message = format!("v{}", env!("CARGO_PKG_VERSION"));
    ctx.reply(message).await?;
    Ok(())
}
