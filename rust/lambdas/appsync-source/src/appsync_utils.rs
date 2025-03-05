use std::ops::BitOr;

use lambda_commons_utils::serde_json::Value;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryField {
    GameState,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MutationField {
    StartGame,
    StopGame,
    ResetGame,
    RegisterNewPlayer,
    UpdatePlayerName,
    RemovePlayer,
    Click,
    ReportLatency,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionField {
    UpdatedPlayer,
    RemovedPlayer,
    UpdatedGameStatus,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(tag = "parentTypeName", content = "fieldName")]
pub enum Operation {
    Query(QueryField),
    Mutation(MutationField),
    Subscription(SubscriptionField),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AppSyncAuthStrategy {
    Allow,
    Deny,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AppSyncIdentity {
    pub sub: String,
    pub username: String,
    pub issuer: String,
    #[serde(rename = "defaultAuthStrategy")]
    pub auth_strategy: AppSyncAuthStrategy,
    #[serde(rename = "sourceIp")]
    pub source_ip: Vec<String>,
    pub groups: Vec<String>,
    pub claims: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppSyncEvent {
    pub identity: Option<AppSyncIdentity>,
    pub request: Value,
    pub source: Value,
    pub info: Operation,
    #[serde(rename = "arguments")]
    pub args: Value,
    // Should never be usefull in a Direct Lambda Invocation context
    // pub stash: Value,
    // pub prev: Value,
}

#[derive(Debug, Serialize)]
pub struct AppSyncResponse {
    pub data: Option<Value>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub error: Option<AppSyncError>,
}

#[derive(Debug, Error, Serialize)]
#[serde(rename_all = "camelCase")]
#[error("{error_type}: {error_message}")]
pub struct AppSyncError {
    pub error_type: String,
    pub error_message: String,
}
impl AppSyncError {
    pub fn new(error_type: impl Into<String>, error_message: impl Into<String>) -> Self {
        AppSyncError {
            error_type: error_type.into(),
            error_message: error_message.into(),
        }
    }
}

impl BitOr for AppSyncError {
    type Output = AppSyncError;
    fn bitor(self, rhs: Self) -> Self::Output {
        AppSyncError {
            error_type: format!("{}|{}", self.error_type, rhs.error_type),
            error_message: format!("{}\n{}", self.error_message, rhs.error_message),
        }
    }
}

impl From<aws_sdk_dynamodb::Error> for AppSyncError {
    fn from(e: aws_sdk_dynamodb::Error) -> Self {
        let meta = aws_sdk_dynamodb::error::ProvideErrorMetadata::meta(&e);
        AppSyncError {
            error_type: meta.code().unwrap_or("Unknown").to_owned(),
            error_message: meta.message().unwrap_or_default().to_owned(),
        }
    }
}
