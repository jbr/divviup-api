use crate::client::{HpkeConfig, TaskCreate, TaskMetrics, TaskResponse};
use fastrand::alphanumeric;
use std::iter::repeat_with;
use trillium::{Conn, Handler, Status};
use trillium_api::{api, Json};
use trillium_logger::{dev_formatter, logger};
use trillium_router::router;

pub fn aggregator_api() -> impl Handler {
    (
        logger().with_formatter(("[aggregator mock] ", dev_formatter)),
        router()
            .get("/health", Status::Ok)
            .post("/tasks", api(post_task))
            .delete("/tasks/:task_id", Status::Ok)
            .get("/tasks/:task_id/metrics", api(get_task_metrics)),
    )
}

async fn get_task_metrics(_: &mut Conn, (): ()) -> Json<TaskMetrics> {
    Json(TaskMetrics {
        reports: fastrand::u64(..),
        report_aggregations: fastrand::u64(..),
    })
}

async fn post_task(_: &mut Conn, Json(req): Json<TaskCreate>) -> Json<TaskResponse> {
    Json(TaskResponse {
        task_id: repeat_with(alphanumeric).take(10).collect(),
        aggregator_endpoints: req.aggregator_endpoints,
        query_type: req.query_type,
        vdaf: req.vdaf,
        role: req.role,
        vdaf_verify_keys: vec![repeat_with(alphanumeric).take(10).collect()],
        max_batch_query_count: req.max_batch_query_count,
        task_expiration: req.task_expiration,
        report_expiry_age: None,
        min_batch_size: req.min_batch_size,
        time_precision: req.time_precision,
        tolerable_clock_skew: 60,
        collector_hpke_config: HpkeConfig {
            id: 1,
            kem_id: 1,
            kdf_id: 1,
            aead_id: 1,
            public_key: b"this is a public key".to_vec(),
        },
        aggregator_auth_tokens: vec![],
        collector_auth_tokens: vec![],
        aggregator_hpke_configs: [(
            1,
            HpkeConfig {
                id: 1,
                kem_id: 1,
                kdf_id: 1,
                aead_id: 1,
                public_key: b"this is a public key".to_vec(),
            },
        )]
        .into_iter()
        .collect(),
    })
}