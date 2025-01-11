use crate::command::{CommandError, Data};
use anyhow::Context;
use tokio_stream::StreamExt;

/// Display a list of the summoners that the guild is subscribed to
#[poise::command(slash_command, guild_only, ephemeral)]
pub async fn list(ctx: poise::Context<'_, Data, CommandError>) -> Result<(), CommandError> {
    let guild_id = ctx.guild_id().context("Trying to get guild ID")?;

    // Get raw data from DB
    let followed_data = ctx
        .data()
        .db_handler
        .get_guild_follows(guild_id.into())
        .await?;

    if followed_data.is_empty() {
        ctx.reply("No followed summoners.").await?;
        return Ok(());
    }

    // Format the message
    // TODO: Examine better ways to handle this
    let mut message = String::from("**FOLLOWED SUMMONERS**\n**-------------------------**\n");
    let lines = tokio_stream::iter(followed_data).then(|guild_following| async move {
        let summoner = ctx
            .data()
            .db_handler
            .get_summoner(&guild_following.puuid)
            .await?
            .context("Failed to get summoner matching PUUID from database")?;
        let summoner_name = format!("{}#{}", summoner.game_name, summoner.tag);
        Ok::<String, anyhow::Error>(summoner_name)
    });
    tokio::pin!(lines);

    let mut index = 1;
    while let Some(line) = lines.next().await {
        message += &format!("**{})** {}\n", index, line?);
        index += 1;
    }

    // Send the message
    ctx.reply(message).await?;
    Ok(())
}
