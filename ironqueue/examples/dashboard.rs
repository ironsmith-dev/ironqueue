//! The README's standalone dashboard server, compiled with the crate so the quickstart cannot
//! drift from the API. Run it against the Docker Compose Postgres:
//!
//! `DATABASE_URL=postgres://ironqueue:ironqueue@localhost:5439/ironqueue \
//!  IRONQUEUE_DASHBOARD_PASSWORD=local-password cargo run --example dashboard`

use ironqueue::{Dashboard, Queue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let queue = Queue::connect(&database_url).await?;

    Dashboard::new([queue])
        .basic_auth("admin", std::env::var("IRONQUEUE_DASHBOARD_PASSWORD")?)
        .serve_on("localhost", 8080)
        .run()
        .await?;

    Ok(())
}
