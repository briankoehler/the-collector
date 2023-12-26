use crate::server::Server;
use dotenvy::dotenv;
use match_fetcher::MatchFetcher;

mod db;
mod match_fetcher;
mod server;

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
    let fetcher = MatchFetcher::new(&mut conn);
    fetcher.start().await;
}
