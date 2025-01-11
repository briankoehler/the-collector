use crate::command::{CommandError, Data};

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
