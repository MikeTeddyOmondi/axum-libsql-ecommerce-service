#![allow(unused)]

mod db;
mod rest;

use crate::db::init_db;
use anyhow::Result;
use axum::{Extension, Router};
use libsql::Connection;
// use sqlx::SqlitePool;

/// Build the overall web service router.
/// Constructing the router in a function makes it easy to re-use in unit tests.
fn router(connection_pool: Connection) -> Router {
    Router::new()
        // Nest service allows you to attach another router to a URL base.
        // "/" inside the service will be "/books" to the outside world.
        .nest_service("/posts", rest::posts_service())
        // Add the connection pool as a "layer", available for dependency injection.
        .layer(Extension(connection_pool))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env if available
    dotenvy::dotenv().ok();

    // Initialize the database and obtain a connection pool
    let connection = init_db().await?;

    // Initialize the Axum routing service
    let app: Router = router(connection);

    // Define the address to listen on (everything)
    // let addr = SocketAddr::from(([0, 0, 0, 0], 3001));

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3838").await.unwrap();

    println!("Server listening...");

    axum::serve(listener, app).await?;

    Ok(())
}
