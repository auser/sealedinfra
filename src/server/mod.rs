use std::sync::Arc;

use app_state::AppState;
use axum::http::{header::CONTENT_TYPE, Method};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::{error::SealedResult, settings::ServerArgs};

mod app_state;
pub(crate) mod git;
mod models;
pub(crate) mod repo;
mod routes;
pub(crate) mod utils;

#[derive(Debug)]
pub struct Server {
    args: ServerArgs,
}

impl Server {
    pub async fn new(args: ServerArgs) -> Self {
        Self { args }
    }

    pub async fn run(&self) -> SealedResult<()> {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([CONTENT_TYPE]);

        let app_state = AppState::new().await?;
        let shared_state = Arc::new(app_state);

        let app = routes::routes(shared_state);
        let app = app.layer(cors);

        println!(
            "Server started successfully at http://0.0.0.0:{}",
            self.args.port
        );

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.args.port))
            .await
            .unwrap();
        axum::serve(listener, app).await?;

        Ok(())
    }
}
