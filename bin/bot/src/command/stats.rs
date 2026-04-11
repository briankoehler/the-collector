use crate::command::{CommandError, Data};
use anyhow::Context;
use indoc::formatdoc;
use the_collector_db::{model::SummonerAggregateStats, types::PgInterval};

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
    let total_minutes = interval_to_minutes(&data.total_duration);
    let total_time_dead_minutes = interval_to_minutes(&data.total_time_dead);

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
        matches = data.num_matches.unwrap_or(0), // TODO: N/A instead of 0
        playtime = total_minutes,
        total_dead = total_time_dead_minutes,
        kills = data.kills,
        deaths = data.deaths,
        assists = data.assists,
        death_rate = total_minutes / data.deaths as f64,
        dead_percent = (total_time_dead_minutes / total_minutes) * 100.0,
    }
}

fn interval_to_minutes(interval: &PgInterval) -> f64 {
    interval.days as f64 * 24.0 * 60.0
        + interval.microseconds as f64 / 1_000_000.0 / 60.0
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
            num_matches: Some(200),
            kills: 900,
            deaths: 560,
            assists: 1300,
            total_duration: PgInterval { months: 0, days: 0, microseconds: 15000 * 1_000_000 },
            total_time_dead: PgInterval { months: 0, days: 0, microseconds: 5000 * 1_000_000 },
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
