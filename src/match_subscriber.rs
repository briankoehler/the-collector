use crate::db::{
    model::{Match, MatchStat},
    schema::{match_, match_stats},
};
use async_trait::async_trait;
use diesel::{Insertable, RunQueryDsl, SqliteConnection};
use tokio::{io::AsyncWriteExt, net::UnixStream};

#[async_trait]
pub trait MatchSubscribe {
    async fn handle_match(&mut self, match_data: &Match);
    async fn handle_match_stats(&mut self, match_data: &MatchStat);
}

pub struct SocketSubscriber(pub UnixStream);

#[async_trait]
impl MatchSubscribe for SocketSubscriber {
    async fn handle_match(&mut self, _match_data: &Match) {
        println!("Ignoring match data (not used by socket)")
    }

    async fn handle_match_stats(&mut self, match_stats: &MatchStat) {
        let serialized_data = serde_json::to_string(&match_stats).unwrap();
        let serialized_data = serialized_data.as_bytes();
        self.0.write_all(serialized_data).await.unwrap();
    }
}

pub struct DatabaseSubscriber<'a>(pub &'a mut SqliteConnection);

#[async_trait]
impl<'a> MatchSubscribe for DatabaseSubscriber<'a> {
    async fn handle_match_stats(&mut self, match_stats: &MatchStat) {
        match_stats
            .insert_into(match_stats::table)
            .get_result::<MatchStat>(self.0)
            .unwrap();

        println!(
            "Inserted match stats: {:?}",
            serde_json::to_string(match_stats)
        );
    }

    async fn handle_match(&mut self, match_data: &Match) {
        let inserted_match_count = match_data
            .insert_into(match_::table)
            .on_conflict_do_nothing()
            .execute(self.0)
            .unwrap();
        println!("Inserted match count: {inserted_match_count}");
    }
}
