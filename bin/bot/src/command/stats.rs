use crate::command::{CommandError, Data};
use anyhow::Context;
use indoc::formatdoc;
use the_collector_db::model::SummonerAggregateStats;

/// Display statistics of the provided summoner
#[poise::command(slash_command, guild_only)]
pub async fn stats(
    ctx: poise::Context<'_, Data, CommandError>,
    #[description = "Summoner Name"] name: String,
    #[description = "Summoner Tag"] tag: String,
) -> Result<(), CommandError> {
    let db_handler = &ctx.data().db_handler;

    let stats_data = db_handler
        .get_summoner_stats(&name, &tag)
        .await?
        .context("No data for **{name}#{tag}** found.")?;

    let message = create_message(&stats_data);
    ctx.reply(message).await?;
    Ok(())
}

fn create_message(data: &SummonerAggregateStats) -> String {
    let total_minutes = data.total_duration as f64 / 60.0;
    let total_time_dead = data.total_time_dead as f64 / 60.0;

    formatdoc! {"
        **{game_name}#{tag_line} Stats**
 
        **Total Matches:** {matches}
        **Total Playtime:** {playtime} minutes
        **Total Time Spent Dead:** {total_dead:.2} minutes

        **Total Kills:** {kills}
        **Total Deaths:** {deaths}
        **Total Assists:** {assists}

        **Minutes per Death:** {death_rate:.2} minutes
        **Percentage Time Spend Dead:** {dead_percent:.2}%
        ",
        game_name = data.game_name,
        tag_line = data.tag,
        matches = data.num_matches,
        playtime = total_minutes,
        total_dead = total_time_dead,
        kills = data.kills,
        deaths = data.deaths,
        assists = data.assists,
        death_rate = total_minutes / data.deaths as f64,
        dead_percent = (total_time_dead / total_minutes) * 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use the_collector_db::model::SummonerAggregateStats;

    #[test]
    fn test_create_message() {
        let input_data = SummonerAggregateStats {
            game_name: String::from("riot"),
            tag: String::from("games"),
            num_matches: 200,
            kills: 900,
            deaths: 560,
            assists: 1300,
            total_duration: 15000,
            total_time_dead: 5000,
        };

        let message = create_message(&input_data);
        let mut lines = message.lines();
        assert_eq!(lines.next().unwrap(), "**riot#games Stats**");
        assert_eq!(lines.next().unwrap(), "");
        assert_eq!(lines.next().unwrap(), "**Total Matches:** 200");
        assert_eq!(lines.next().unwrap(), "**Total Playtime:** 250 minutes");
        assert_eq!(
            lines.next().unwrap(),
            "**Total Time Spent Dead:** 83.33 minutes"
        );
        assert_eq!(lines.next().unwrap(), "");
        assert_eq!(lines.next().unwrap(), "**Total Kills:** 900");
        assert_eq!(lines.next().unwrap(), "**Total Deaths:** 560");
        assert_eq!(lines.next().unwrap(), "**Total Assists:** 1300");
        assert_eq!(lines.next().unwrap(), "");
        assert_eq!(lines.next().unwrap(), "**Minutes per Death:** 0.45 minutes");
        assert_eq!(
            lines.next().unwrap(),
            "**Percentage Time Spend Dead:** 33.33%"
        );
        assert!(lines.next().is_none());
    }
}
