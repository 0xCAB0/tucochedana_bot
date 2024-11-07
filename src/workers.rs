use crate::DATABASE_URL;
use crate::TASK_NAME;

use fang::asynk::async_queue::AsyncQueue;
use fang::asynk::async_worker_pool::AsyncWorkerPool;
//use fang::AsyncQueueable;
use fang::FangError;
use fang::NoTls;
use fang::SleepParams;

use std::time::Duration;

const MAX_WORKERS: u32 = 15u32;

pub async fn start_workers() -> Result<AsyncQueue<NoTls>, FangError> {
    let mut queue: AsyncQueue<NoTls> = AsyncQueue::builder()
        .uri(DATABASE_URL.to_string())
        .max_pool_size(MAX_WORKERS)
        .build();

    queue.connect(NoTls).await.unwrap();

    let params = SleepParams {
        sleep_period: Duration::from_millis(250),
        max_sleep_period: Duration::from_secs(60_u64),
        min_sleep_period: Duration::from_secs(0),
        sleep_step: Duration::from_millis(250),
    };

    let mut pool_scheduled_fetch: AsyncWorkerPool<AsyncQueue<NoTls>> = AsyncWorkerPool::builder()
        .number_of_workers(MAX_WORKERS)
        .sleep_params(params)
        .queue(queue.clone())
        .task_type(TASK_NAME)
        .build();

    pool_scheduled_fetch.start().await;

    Ok(queue)
}
