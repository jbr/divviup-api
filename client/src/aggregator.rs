use crate::Protocol;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Role {
    Leader,
    Helper,
    Either,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Aggregator {
    pub id: Uuid,
    // an account_id of None indicates a shared Aggregator
    pub account_id: Option<Uuid>,
    #[serde(with = "::time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "::time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    // a deleted_at of Some indicates a tombstoned/inactivated Aggregator
    #[serde(default, with = "::time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub role: Role,
    pub name: String,
    pub dap_url: Url,
    pub api_url: Url,
    pub is_first_party: bool,
    pub vdafs: Vec<String>,
    pub query_types: Vec<String>,
    pub protocol: Protocol,
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct NewAggregator {
    pub name: String,
    pub api_url: Url,
    pub bearer_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct NewSharedAggregator {
    pub name: String,
    pub api_url: Url,
    pub is_first_party: bool,
    pub bearer_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
// Currently, Janus collector authentication tokens are always bearer tokens.
pub enum CollectorAuthenticationToken {
    Bearer { token: String },
}
