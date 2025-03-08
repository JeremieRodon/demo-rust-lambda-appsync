mod id;
use std::{collections::HashMap, ops::BitOr};

use serde_json::{self, Value};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

pub use id::ID;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AppSyncAuthStrategy {
    Allow,
    Deny,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppSyncEventInfo<O> {
    #[serde(flatten)]
    pub operation: O,
    #[serde(rename = "selectionSetGraphQL")]
    pub selection_set_graphql: String,
    #[serde(rename = "selectionSetList")]
    pub selection_set_list: Vec<String>,
    pub variables: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppSyncEvent<O> {
    pub identity: Option<AppSyncIdentity>,
    pub request: Value,
    pub source: Value,
    pub info: AppSyncEventInfo<O>,
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

pub fn arg_from_json<T: DeserializeOwned>(
    args: &mut serde_json::Value,
    arg_name: &'static str,
) -> Result<T, AppSyncError> {
    serde_json::from_value(
        args.get_mut(arg_name)
            .unwrap_or(&mut serde_json::Value::Null)
            .take(),
    )
    .map_err(|e| {
        AppSyncError::new(
            "InvalidArgs",
            format!("Argument \"{arg_name}\" is not the expected format ({e})"),
        )
    })
}

pub fn res_to_json<T: Serialize>(res: T) -> serde_json::Value {
    serde_json::to_value(res).expect("AppSync schema objects are JSON compatible")
}
