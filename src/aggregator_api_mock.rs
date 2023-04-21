use crate::clients::aggregator_client::api_types::{
    HpkeConfig, TaskCreate, TaskIds, TaskMetrics, TaskResponse,
};
use fastrand::alphanumeric;
use querystrong::QueryStrong;
use std::iter::repeat_with;
use trillium::{Conn, Handler, Status};
use trillium_api::{api, Json};
use trillium_logger::{dev_formatter, logger};
use trillium_router::router;
use uuid::Uuid;

pub fn aggregator_api() -> impl Handler {
    (
        logger().with_formatter(("[aggregator mock] ", dev_formatter)),
        router()
            .post("/tasks", api(post_task))
            .get("/task_ids", api(task_ids))
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

async fn post_task(_: &mut Conn, Json(task_create): Json<TaskCreate>) -> Json<TaskResponse> {
    Json(task_response(task_create))
}

pub fn task_response(task_create: TaskCreate) -> TaskResponse {
    TaskResponse {
        task_id: repeat_with(alphanumeric).take(10).collect(),
        aggregator_endpoints: task_create.aggregator_endpoints,
        query_type: task_create.query_type,
        vdaf: task_create.vdaf,
        role: task_create.role,
        vdaf_verify_keys: vec![repeat_with(alphanumeric).take(10).collect()],
        max_batch_query_count: task_create.max_batch_query_count,
        task_expiration: task_create.task_expiration,
        report_expiry_age: None,
        min_batch_size: task_create.min_batch_size,
        time_precision: task_create.time_precision,
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
    }
}

async fn task_ids(conn: &mut Conn, (): ()) -> Result<Json<TaskIds>, Status> {
    let query = QueryStrong::parse(conn.querystring()).map_err(|_| Status::InternalServerError)?;
    match query.get_str("pagination_token") {
        None => Ok(Json(TaskIds {
            task_ids: std::iter::repeat_with(|| Uuid::new_v4().to_string())
                .take(10)
                .collect(),
            pagination_token: Some("second".into()),
        })),

        Some("second") => Ok(Json(TaskIds {
            task_ids: std::iter::repeat_with(|| Uuid::new_v4().to_string())
                .take(10)
                .collect(),
            pagination_token: Some("last".into()),
        })),

        _ => Ok(Json(TaskIds {
            task_ids: std::iter::repeat_with(|| Uuid::new_v4().to_string())
                .take(5)
                .collect(),
            pagination_token: None,
        })),
    }
}
