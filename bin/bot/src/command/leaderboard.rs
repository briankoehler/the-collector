use crate::command::{CommandError, Data};
use crate::ddragon::GameVersion;
use anyhow::Context;
use tokio_stream::StreamExt;

const DEFAULT_LEADERBOARD_SIZE: usize = 10;

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
