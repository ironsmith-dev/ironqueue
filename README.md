# IronQueue

Async and cron jobs for Rust, backed by PostgreSQL 18+.

## Features

- Turn async functions into background jobs with `#[ironqueue::job]`.
- Schedule cron jobs with `#[ironqueue::cron]`.
- Retry, delay, prioritize, deduplicate, and wait for jobs.
- Inspect queues, workers, and jobs in the built-in dashboard.

## Enqueueing Jobs

Define a job and start a worker:

```rust
use ironqueue::{Queue, Worker};
use serde::{Deserialize, Serialize};

// The job's input, taken as the handler function's first parameter (must be JSON-serializable).
#[derive(Serialize, Deserialize)]
pub struct Email {
    pub address: String,
}

// The job's output, returned by the handler function (must be JSON-serializable).
#[derive(Serialize, Deserialize)]
pub struct Receipt {
    pub address: String,
}

// Define a background job.
#[ironqueue::job(
    // Job name, at most 255 bytes (optional; default: function name).
    name = "deliver_email",
    // Total attempts including the initial run (optional; default: 1).
    max_attempts = 5,
    // Max duration of each attempt in milliseconds (optional; default: 10,000; 0 disables timeout).
    timeout_ms = 30_000,
    // Result retention in milliseconds (optional; default: 600,000; 0 deletes immediately).
    result_ttl_ms = 3_600_000,
    // Base retry delay in milliseconds (optional; default: 0).
    retry_delay_ms = 500,
    // Max exponential backoff in milliseconds (optional; default: disabled).
    // Backoff applies full jitter: each retry waits a uniformly random duration
    // up to the bound, so the delays above are ceilings rather than fixed waits.
    max_backoff_ms = 60_000,
    // Dequeue priority; lower values run first (optional; default: 0).
    priority = -10,
)]
pub async fn send_email(args: Email) -> anyhow::Result<Receipt> {
    println!("emailing {}", args.address);
    Ok(Receipt { address: args.address })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let queue = Queue::connect(&database_url).await?;

    Worker::builder(queue)
        .register_job(send_email)
        .run()
        .await?;

    Ok(())
}
```

In another process, enqueue the job:

```rust
use std::time::Duration;

use crate::{send_email, Email, Receipt};
use ironqueue::{EnqueueResult, JobHandle, Queue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let queue = Queue::connect(&database_url).await?;

    // Enqueue without waiting for the job to finish.
    let job1 = send_email::job(Email { address: "user1@example.com".into() })
        .dedupe_key("welcome:user@example.com")
        .delay(Duration::from_secs(5));
    let result: EnqueueResult<JobHandle<send_email>> = queue.enqueue(job1).await?;
    println!("job id: {}", result.job_id());

    // Enqueue and wait for the job to finish.
    let job2 = send_email::job(Email { address: "user2@example.com".into() });
    let receipt: Receipt = queue
        .enqueue_and_wait(job2, Some(Duration::from_secs(30)))
        .await?;
    println!("receipt for: {}", receipt.address);

    Ok(())
}
```

## Cron Jobs

Define cron jobs to run on a recurring schedule:

```rust
use ironqueue::{JobContext, Queue, Worker};

// Cron jobs have no payload. Parameters are context extractors.
#[ironqueue::cron(
    // Five-field crontab schedule (`minute hour day-of-month month day-of-week`) in UTC. Sunday is `0` or `7`.
    "0 * * * *",
    // Revision; increment after changes, highest wins across workers (optional; default: 0).
    // The stored schedule is compared verbatim, so *any* textual edit needs a bump — including a
    // semantically equivalent one like `SUN` to `0`, or a whitespace change. Editing the expression
    // without bumping the revision leaves workers disagreeing and disables the cron.
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
    // Backoff applies full jitter, as above.
    max_backoff_ms = 60_000,
    // Dequeue priority; lower values run first (optional; default: 0).
    priority = 10,
)]
async fn collect_hourly_metrics(ctx: JobContext) -> anyhow::Result<()> {
    let queued = ctx.queue().counts().await?.queued;
    println!("{queued} job(s) queued");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let queue = Queue::connect(&database_url).await?;

    Worker::builder(queue)
        .register_cron(collect_hourly_metrics)
        .run()
        .await?;

    Ok(())
}
```

For schedules loaded at runtime, define a regular `#[ironqueue::job]` and use `WorkerBuilder::schedule_cron`:

```rust
use ironqueue::{Queue, Worker};

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
        .schedule_cron("0 3 * * *", cleanup::job(()))
        .run()
        .await?;

    Ok(())
}
```

## Dashboard

Run the built-in web dashboard as a standalone server to inspect queues, workers, and jobs:

```rust
use ironqueue::{Dashboard, Queue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let queue = Queue::connect(&database_url).await?;

    Dashboard::new([queue])
        .basic_auth("admin", std::env::var("IRONQUEUE_DASHBOARD_PASSWORD")?)
        .secure_cookies(false) // only for direct HTTP on a trusted network
        .serve_on("localhost", 8080)
        .run()
        .await?;

    Ok(())
}
```

## Technical Notes

- At-least-once delivery (requires PostgreSQL to have `fsync` and `synchronous_commit` enabled).
- Job payloads, job results, job metadata, worker stats, and worker metadata are each limited to 1 MiB of serialized
  JSON. IronQueue rejects oversized documents before writing anything.
- Every queue connection checks IronQueue's migration history and applies missing migrations automatically. A current
  history only needs read access to `ironqueue.migrations`; it does not run DDL or take the migrator's advisory lock.
  Migration history does not detect or repair a table, index, or other object changed manually after its migration ran.
  The first connection to a new database, and the first connection after an upgrade, needs permission to create or
  update IronQueue's schema.
- A least-privilege deployment can install the published migration command with `cargo install ironqueue`, then run
  `DATABASE_URL=postgres://schema-owner:password@host/database ironqueue-migrate` as its release step. The application
  can then start with its normal restricted database role.
