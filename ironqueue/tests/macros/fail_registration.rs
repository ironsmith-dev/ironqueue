#[ironqueue::job]
async fn ordinary(_: ()) {}

#[ironqueue::cron("* * * * *")]
async fn scheduled() {}

fn register_job_rejects_cron(builder: ironqueue::WorkerBuilder) {
    let _ = builder.register_job(scheduled);
}

fn register_cron_rejects_job(builder: ironqueue::WorkerBuilder) {
    let _ = builder.register_cron(ordinary);
}

fn schedule_cron_rejects_cron(builder: ironqueue::WorkerBuilder) {
    let _ = builder.schedule_cron("* * * * *", scheduled::job());
}

fn main() {}
