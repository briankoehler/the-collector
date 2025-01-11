use crate::ddragon::DataDragon;
use riven::RiotApi;
use std::sync::Arc;
use the_collector_db::DbHandler;
use tokio::sync::Mutex;

mod about;
mod follow;
mod here;
mod leaderboard;
mod list;
mod stats;
mod unfollow;
mod unhere;

pub use about::about;
pub use follow::follow;
pub use here::here;
pub use leaderboard::leaderboard;
pub use list::list;
pub use stats::stats;
pub use unfollow::unfollow;
pub use unhere::unhere;

type CommandError = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
    pub db_handler: Arc<DbHandler>,
    pub riot_api: RiotApi,
    pub data_dragon: Mutex<DataDragon>,
}
