use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct ID(uuid::Uuid);
impl ID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
impl TryFrom<String> for ID {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(ID(uuid::Uuid::parse_str(&value)?))
    }
}
impl core::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<ID> for String {
    fn from(value: ID) -> Self {
        value.to_string()
    }
}
impl Deref for ID {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
