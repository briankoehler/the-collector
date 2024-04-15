use crate::server::Server;
use dotenvy::dotenv;
use match_fetcher::MatchFetcher;
use match_subscriber::{DatabaseSubscriber, SocketSubscriber};
use tokio::net::UnixStream;

mod db;
mod match_fetcher;
mod match_subscriber;
mod riven_wrapper;
mod server;

const SOCKET_PATH: &str = "/tmp/lyte.socket";

#[tokio::main]
async fn main() {
    dotenv().ok().unwrap();

    // Spawn a tokio task for the running HTTP server that
    // provides access to the database
    // TODO: Provide a DB connection
    tokio::task::spawn(async {
        let server = Server::new();
        server.start().await;
    });

    // Establish a DB connectioon and start the MatchFetcher
    let mut conn = db::establish_connection();
    let riot_api_key = std::env::var("RIOT_API_KEY").unwrap();
    let mut fetcher = MatchFetcher::new(riot_api_key.as_str(), &mut conn);

    // Socket
    let mut socket = SocketSubscriber(UnixStream::connect(SOCKET_PATH).await.unwrap());
    fetcher.add_match_subscriber(&mut socket);

    // Database
    let mut conn = db::establish_connection();
    let mut db_sub = DatabaseSubscriber(&mut conn);
    fetcher.add_match_subscriber(&mut db_sub);

    fetcher.start().await;
}
