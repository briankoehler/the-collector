use crate::command::{CommandError, Data};
use anyhow::Context;
use riven::consts::RegionalRoute;

/// Subscribes the guild to the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn follow(
    ctx: poise::Context<'_, Data, CommandError>,
    #[description = "Summoner Name"] name: String,
    #[description = "Summoner Tag"] tag: String,
) -> Result<(), CommandError> {
    let guild_id = ctx.guild_id().context("Trying to get guild ID")?;
    let db_handler = &ctx.data().db_handler;
    let riot_api = &ctx.data().riot_api;

    // Always query the API to guarantee we're using the PUUID that matches
    // with the summoner with that name and tag at this point in time
    let Some(account) = riot_api
        .account_v1()
        .get_by_riot_id(RegionalRoute::AMERICAS, &name, &tag)
        .await?
    else {
        let message = format!("No summoner exists with name **{name}#{tag}**.");
        ctx.reply(message).await?;
        return Ok(());
    };
    let name = account.game_name.as_ref().unwrap_or(&name);
    let tag = account.tag_line.as_ref().unwrap_or(&tag);

    // Do not proceed if the guild already follows them
    let following = db_handler.get_guild_follows(guild_id.into()).await?;
    if following
        .iter()
        .any(|following| following.puuid == account.puuid)
    {
        let message = format!("Already following **{name}#{tag}**.");
        ctx.reply(message).await?;
        return Ok(());
    }

    // Insert information into database
    db_handler.insert_summoner(&account).await?;
    db_handler
        .insert_guild_following(guild_id.into(), &account.puuid)
        .await?;

    let message = format!("Followed **{name}#{tag}**.");
    ctx.reply(message).await?;
    Ok(())
}
