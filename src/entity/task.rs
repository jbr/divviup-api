use crate::{
    clients::aggregator_client::{api_types::TaskResponse, TaskMetrics},
    entity::{
        account, membership, AccountColumn, Accounts, Aggregator, AggregatorColumn, Aggregators,
        CollectorCredentialColumn, CollectorCredentials,
    },
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ConnectionTrait, DbErr, DeriveEntityModel,
    DerivePrimaryKey, DeriveRelation, EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait,
    Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use validator::{Validate, ValidationError};

pub mod vdaf;
use vdaf::Vdaf;
mod new_task;
pub use new_task::NewTask;
mod update_task;
pub use update_task::UpdateTask;
mod provisionable_task;
pub use provisionable_task::{ProvisionableTask, TaskProvisioningError};

pub const DEFAULT_EXPIRATION_DURATION: Duration = Duration::days(365);

use super::json::Json;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: Uuid,
    pub name: String,
    pub vdaf: Json<Vdaf>,
    pub min_batch_size: i64,
    pub max_batch_size: Option<i64>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub time_precision_seconds: i32,
    pub report_count: i32,
    pub aggregate_collection_count: i32,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub expiration: Option<OffsetDateTime>,
    pub leader_aggregator_id: Uuid,
    pub helper_aggregator_id: Uuid,
    pub collector_credential_id: Uuid,
}

impl Model {
    pub async fn update_metrics(
        self,
        metrics: TaskMetrics,
        db: impl ConnectionTrait,
    ) -> Result<Self, DbErr> {
        let mut task = self.into_active_model();
        task.report_count = ActiveValue::Set(metrics.reports.try_into().unwrap_or(i32::MAX));
        task.aggregate_collection_count =
            ActiveValue::Set(metrics.report_aggregations.try_into().unwrap_or(i32::MAX));
        task.updated_at = ActiveValue::Set(OffsetDateTime::now_utc());
        task.update(&db).await
    }

    pub async fn leader_aggregator(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<super::Aggregator, DbErr> {
        super::Aggregators::find_by_id(self.leader_aggregator_id)
            .one(db)
            .await
            .transpose()
            .ok_or(DbErr::Custom("expected leader aggregator".into()))?
    }

    pub async fn helper_aggregator(&self, db: &impl ConnectionTrait) -> Result<Aggregator, DbErr> {
        Aggregators::find_by_id(self.helper_aggregator_id)
            .one(db)
            .await
            .transpose()
            .ok_or(DbErr::Custom("expected helper aggregator".into()))?
    }

    pub async fn aggregators(&self, db: &impl ConnectionTrait) -> Result<[Aggregator; 2], DbErr> {
        futures_lite::future::try_zip(self.leader_aggregator(db), self.helper_aggregator(db))
            .await
            .map(|(leader, helper)| [leader, helper])
    }

    pub async fn first_party_aggregator(
        &self,
        db: &impl ConnectionTrait,
    ) -> Result<Option<Aggregator>, DbErr> {
        Ok(self
            .aggregators(db)
            .await?
            .into_iter()
            .find(|agg| agg.is_first_party))
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Accounts",
        from = "Column::AccountId",
        to = "AccountColumn::Id"
    )]
    Account,

    #[sea_orm(
        belongs_to = "Aggregators",
        from = "Column::HelperAggregatorId",
        to = "AggregatorColumn::Id"
    )]
    HelperAggregator,

    #[sea_orm(
        belongs_to = "Aggregators",
        from = "Column::LeaderAggregatorId",
        to = "AggregatorColumn::Id"
    )]
    LeaderAggregator,

    #[sea_orm(
        belongs_to = "CollectorCredentials",
        from = "Column::CollectorCredentialId",
        to = "CollectorCredentialColumn::Id"
    )]
    CollectorCredential,
}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<membership::Entity> for Entity {
    fn to() -> RelationDef {
        account::Relation::Memberships.def()
    }

    fn via() -> Option<RelationDef> {
        Some(account::Relation::Tasks.def().rev())
    }
}

impl Related<CollectorCredentials> for Entity {
    fn to() -> RelationDef {
        Relation::CollectorCredential.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
