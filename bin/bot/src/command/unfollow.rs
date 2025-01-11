use crate::command::{CommandError, Data};
use anyhow::Context;

/// Unsubscribe the guild from the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn unfollow(
    ctx: poise::Context<'_, Data, CommandError>,
    #[description = "Summoner Name"] name: String,
    #[description = "Summoner Tag"] tag: String,
) -> Result<(), CommandError> {
    let guild_id = ctx.guild_id().context("Trying to get guild ID")?;

    let Some(summoner) = ctx
        .data()
        .db_handler
        .get_summoner_by_name(&name, &tag)
        .await?
    else {
        let message = format!("No summoner being followed with name {name}#{tag}");
        ctx.reply(message).await?;
        return Ok(());
    };
    ctx.data()
        .db_handler
        .delete_guild_following(guild_id.into(), &summoner.puuid)
        .await?;

    let message = format!("Stopped following {}#{}.", summoner.game_name, summoner.tag);
    ctx.say(message).await?;
    Ok(())
}
