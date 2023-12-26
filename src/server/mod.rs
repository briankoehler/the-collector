use axum::{
    routing::{get, post, put},
    Router,
};
use routes::account::*;
use routes::match_stats::*;
use routes::r#match::*;

mod routes;

/// Handles HTTP requests to get/set information in the database. Most
/// helpful for debugging/troubleshooting.
pub struct Server {
    router: Router,
}

impl Server {
    /// Creates a server with several routes useful for debugging.
    pub fn new() -> Self {
        let account_router = Router::new()
            .route("/", post(post_account_by_name).put(put_account))
            .route("/all", get(get_accounts))
            .route("/:puuid", get(get_account).delete(delete_account));

        let match_router = Router::new()
            .route("/", put(put_match))
            .route("/all", get(get_matches))
            .route("/:id", get(get_match).delete(delete_match));

        let match_stats_router = Router::new()
            .route("/", put(put_match_stats))
            .route("/all", get(get_match_stats))
            .route("/:id", get(get_match_stat).delete(delete_match_stats));

        let router = Router::new()
            .nest("/account", account_router)
            .nest("/match", match_router)
            .nest("/match_stat", match_stats_router);

        Self { router }
    }

    // TODO: Configurable address
    /// Starts the server at 0.0.0.0:3000
    pub async fn start(self) {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, self.router).await.unwrap();
    }
}
