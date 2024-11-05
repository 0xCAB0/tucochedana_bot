use crate::consts::{SCHEDULED_TASK_TYPE, TASK_TYPE};

use crate::CERT_FILE_TLS;
use crate::DATABASE_URL;
use crate::MAX_WORKERS;
use fang::asynk::async_queue::AsyncQueue;
use fang::asynk::async_worker_pool::AsyncWorkerPool;
//use fang::AsyncQueueable;
use fang::FangError;
//use fang::NoTls;
use fang::SleepParams;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use std::time::Duration;

pub async fn start_workers() -> Result<(), FangError> {
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder
        .set_ca_file(&*CERT_FILE_TLS)
        .expect("Cert file not found");
    let connector = MakeTlsConnector::new(builder.build());

    let mut queue: AsyncQueue<MakeTlsConnector> = AsyncQueue::builder()
        .uri(DATABASE_URL.clone())
        .max_pool_size(*MAX_WORKERS)
        .build();

    queue.connect(connector).await.unwrap();

    let params = SleepParams {
        sleep_period: Duration::from_millis(250),
        max_sleep_period: Duration::from_secs(60_u64),
        min_sleep_period: Duration::from_secs(0),
        sleep_step: Duration::from_millis(250),
    };

    let mut pool: AsyncWorkerPool<AsyncQueue<MakeTlsConnector>> = AsyncWorkerPool::builder()
        .number_of_workers(*MAX_WORKERS + 20)
        .sleep_params(params.clone())
        .queue(queue.clone())
        .task_type(TASK_TYPE)
        .build();

    let mut pool_scheduled_fetch: AsyncWorkerPool<AsyncQueue<MakeTlsConnector>> =
        AsyncWorkerPool::builder()
            .number_of_workers(*MAX_WORKERS)
            .sleep_params(params)
            .queue(queue.clone())
            .task_type(SCHEDULED_TASK_TYPE)
            .build();

    pool.start().await;
    pool_scheduled_fetch.start().await;
    Ok(())
}
