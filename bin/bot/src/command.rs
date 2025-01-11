use crate::ddragon::{DataDragon, GameVersion};
use anyhow::Context;
use riven::{consts::RegionalRoute, RiotApi};
use std::sync::Arc;
use the_collector_db::DbHandler;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

const DEFAULT_LEADERBOARD_SIZE: usize = 10;

type CommandError = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
    pub db_handler: Arc<DbHandler>,
    pub riot_api: RiotApi,
    pub data_dragon: Mutex<DataDragon>,
}

/// Displays a leaderboard of the top ints
#[poise::command(slash_command, guild_only)]
pub async fn leaderboard(
    ctx: poise::Context<'_, Data, CommandError>,
    #[description = "Leaderboard size to view"]
    #[max = 20]
    count: Option<usize>,
) -> Result<(), CommandError> {
    let guild_id = ctx.guild_id().context("Trying to get guild ID")?;
    ctx.defer().await?;

    // Get leaderboard data from the database
    let leaderboard_data = ctx
        .data()
        .db_handler
        .get_leaderboard(guild_id.into(), count.unwrap_or(DEFAULT_LEADERBOARD_SIZE))
        .await?;

    if leaderboard_data.is_empty() {
        ctx.reply("No leaderboard matches yet.").await?;
        return Ok(());
    }

    let leaderboard_data_len = leaderboard_data.len();
    // Format the leaderboard message
    // TODO: Examine better ways to handle this
    let mut message = String::from("**INT LEADERBOARD**\n**-------------------------**\n");
    let lines = tokio_stream::iter(leaderboard_data).then(|summoner_match| async move {
        // TODO: Get summoner and match data in same query
        let summoner = ctx
            .data()
            .db_handler
            .get_summoner(&summoner_match.puuid)
            .await?
            .context("No summoner found with matching PUUID")?;
        let match_info = ctx
            .data()
            .db_handler
            .get_match(&summoner_match.match_id)
            .await?
            .context("No match found with matching ID")?;

        let version = GameVersion(match_info.game_version.clone())
            .to_data_dragon_version()
            .await?;
        let summoner_name = format!("{}#{}", summoner.game_name, summoner.tag);
        let champion_name = ctx
            .data()
            .data_dragon
            .lock()
            .await
            .get_champion_name(&version, summoner_match.champion_id as u16)
            .await?
            .context("No champion with ID found")?;
        Ok::<String, anyhow::Error>(format!(
            "{}/{}/{} - {} ({})",
            summoner_match.kills,
            summoner_match.deaths,
            summoner_match.assists,
            summoner_name,
            champion_name,
        ))
    });
    tokio::pin!(lines);

    let mut index = 1;
    while let Some(line) = lines.next().await {
        message += &format!("**{})** {}\n", index, line?);
        index += 1;
    }

    if let Some(provided_count) = count {
        if leaderboard_data_len < provided_count {
            message += &format!(
                "*Not enough matches played to fill a leaderboard of {provided_count} yet.*\n"
            );
        }
    }

    // Send the message
    ctx.reply(message).await?;
    Ok(())
}

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

/// Display statistics of the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn stats(
    ctx: poise::Context<'_, Data, CommandError>,
    #[description = "Summoner Name"] _name: String,
    #[description = "Summoner Tag"] _tag: String,
) -> Result<(), CommandError> {
    // TODO
    ctx.say("Coming soon").await?;
    Ok(())
}

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

/// Display a list of the summoners that the guild is subscribed to
#[poise::command(slash_command, guild_only)]
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

/// Display information about the bot (e.g. version)
#[poise::command(slash_command, guild_only)]
pub async fn about(ctx: poise::Context<'_, Data, CommandError>) -> Result<(), CommandError> {
    let message = format!("v{}", env!("CARGO_PKG_VERSION"));
    ctx.reply(message).await?;
    Ok(())
}
