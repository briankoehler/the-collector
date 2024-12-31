use anyhow::Context;
use riven::RiotApi;
use std::sync::Arc;
use the_collector_db::DbHandler;
use tokio_stream::StreamExt;

const LEADERBOARD_SIZE: usize = 10;

pub struct Data {
    pub db_handler: Arc<DbHandler>,
    pub riot_api: RiotApi,
}

// TODO: Support optionally specified leaderboard size
/// Displays a leaderboard of the top ints
#[poise::command(slash_command, guild_only)]
pub async fn leaderboard(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = ctx.guild_id().expect("Called in guild context only");

    // Get leaderboard data from the database
    let leaderboard_data = ctx
        .data()
        .db_handler
        .get_leaderboard::<LEADERBOARD_SIZE>(guild_id.into())
        .await?;

    // Format the leaderboard message
    // TODO: Examine better ways to handle this
    let mut message = String::from("**INT LEADERBOARD**\n**-------------------------**\n");
    let lines = tokio_stream::iter(leaderboard_data).then(|summoner_match| async move {
        let summoner = ctx
            .data()
            .db_handler
            .get_summoner(&summoner_match.puuid)
            .await
            .unwrap()
            .unwrap();
        let summoner_name = format!("{}#{}", summoner.game_name, summoner.tag);
        format!(
            "{}/{}/{} - {} ({})",
            summoner_match.kills,
            summoner_match.deaths,
            summoner_match.assists,
            summoner_name,
            summoner_match.champion_id // TODO: Convert to champion name
        )
    });
    tokio::pin!(lines);

    let mut index = 1;
    while let Some(line) = lines.next().await {
        message += &format!("**{})** {}\n", index, line);
        index += 1;
    }

    // Send the message
    ctx.reply(message).await?;
    Ok(())
}

/// Subscribes the guild of the current context to the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn follow(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    #[description = "Summoner Name"] name: String,
    #[description = "Summoner Tag"] tag: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = ctx.guild_id().expect("Called in guild context only");

    // TODO: Check for puuid in database first
    let account = ctx
        .data()
        .riot_api
        .account_v1()
        .get_by_riot_id(riven::consts::RegionalRoute::AMERICAS, &name, &tag)
        .await?
        .context("No summoner found with this name.")?;

    // TODO: Refactor this to be more efficient and robust
    let following = ctx
        .data()
        .db_handler
        .get_guild_follows(guild_id.into())
        .await?;
    if following
        .iter()
        .any(|following| following.puuid == account.puuid)
    {
        ctx.reply("Already following that summoner.").await?;
        return Ok(());
    }

    ctx.data()
        .db_handler
        .insert_summoner(account.clone())
        .await?;
    ctx.data()
        .db_handler
        .insert_guild_following(guild_id.into(), &account.puuid)
        .await?;

    let message = format!("Followed {name}#{tag}");
    ctx.reply(message).await?;
    Ok(())
}

/// Unsubscribes the guild of the current context to the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn unfollow(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    #[description = "Summoner Name"] name: String,
    #[description = "Summoner Tag"] tag: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guild_id = ctx.guild_id().unwrap();
    todo!();
}

/// Display statistics of the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn stats(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    #[description = "Summoner Name"] name: String,
    #[description = "Summoner Tag"] tag: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guild_id = ctx.guild_id().unwrap();
    todo!();
}

/// Set the channel that notifications are sent to
#[poise::command(slash_command, guild_only)]
pub async fn here(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = ctx.guild_id().expect("Called in guild context only");
    let channel_id = ctx.channel_id();
    ctx.data()
        .db_handler
        .update_channel(guild_id.into(), channel_id.into())
        .await?;
    let message = format!(
        "Setting notification channel to **{}**.",
        ctx.guild_channel().await.unwrap().name
    );
    ctx.reply(message).await?;
    Ok(())
}

/// Set the channel that notifications are sent to
#[poise::command(slash_command, guild_only)]
pub async fn unhere(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    todo!("Unset the notification channel")
}

/// Display a list of the summoners that the guild of the current context is
/// subscribed to
#[poise::command(slash_command, guild_only)]
pub async fn list(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guild_id = ctx.guild_id().expect("Called in guild context only");

    // Get raw data from DB
    let followed_data = ctx
        .data()
        .db_handler
        .get_guild_follows(guild_id.into())
        .await?;

    if followed_data.len() == 0 {
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
            .await
            .unwrap()
            .unwrap();
        let summoner_name = format!("{}#{}", summoner.game_name, summoner.tag);
        summoner_name
    });
    tokio::pin!(lines);

    let mut index = 1;
    while let Some(line) = lines.next().await {
        message += &format!("**{})** {}\n", index, line);
        index += 1;
    }

    // Send the message
    ctx.reply(message).await?;
    Ok(())
}

/// Display information about the bot (e.g. version)
#[poise::command(slash_command, guild_only)]
pub async fn about(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guild_id = ctx.guild_id().unwrap();
    todo!();
}
