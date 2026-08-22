//! The README's "Cron Jobs" worker, compiled with the crate so the quickstart cannot drift from
//! the API: one compile-time `#[ironqueue::cron]` schedule and one runtime schedule via
//! `WorkerBuilder::schedule_cron`. Run it against the Docker Compose Postgres:
//!
//! `DATABASE_URL=postgres://ironqueue:ironqueue@localhost:5439/ironqueue cargo run --example cron`

use ironqueue::{JobContext, Queue, Worker};

// Cron jobs have no payload. Parameters are context extractors.
#[ironqueue::cron(
    // Every hour, in UTC.
    "0 * * * *",
    // Revision; increment after changes, highest wins across workers (optional; default: 0).
    revision = 1,
    // Job name, at most 250 bytes due to the cron dedupe key (optional; default: function name).
    name = "collect_hourly_metrics",
    // Total attempts including the initial run (optional; default: 1).
    max_attempts = 2,
    // Max duration of each attempt in milliseconds (optional; default: 10,000; 0 disables timeout).
    timeout_ms = 120_000,
    // Result retention in milliseconds (optional; default: 600,000; 0 deletes immediately).
    result_ttl_ms = 604_800_000,
    // Base retry delay in milliseconds (optional; default: 0).
    retry_delay_ms = 1_000,
    // Max exponential backoff in milliseconds (optional; default: disabled).
    max_backoff_ms = 60_000,
    // Dequeue priority; lower values run first (optional; default: 0).
    priority = 10,
)]
async fn collect_hourly_metrics(ctx: JobContext) -> anyhow::Result<()> {
    let queued = ctx.queue().counts().await?.queued;
    println!("{queued} job(s) queued");
    Ok(())
}

// For schedules loaded at runtime, define a regular job and use `schedule_cron`.
#[ironqueue::job]
async fn cleanup(_: ()) -> anyhow::Result<()> {
    println!("cleaning up");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let queue = Queue::connect(&database_url).await?;

    Worker::builder(queue)
        .register_cron(collect_hourly_metrics)
        .schedule_cron("0 3 * * *", cleanup::job(()))
        .run()
        .await?;

    Ok(())
}
