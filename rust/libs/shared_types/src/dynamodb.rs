use aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_dynamo::Item;

#[derive(Debug, Serialize, Deserialize)]
pub struct MonoItemCore {
    #[serde(rename = "PK")]
    pk: String,
    #[serde(rename = "SK")]
    sk: String,
    #[serde(rename = "_TYPE")]
    r#type: String,
    #[serde(
        rename = "expiration_timestamp",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    expiration_ts: Option<u64>,
}
impl MonoItemCore {
    pub fn new(pk: impl Into<String>, sk: impl Into<String>, r#type: impl Into<String>) -> Self {
        MonoItemCore {
            pk: pk.into(),
            sk: sk.into(),
            r#type: r#type.into(),
            expiration_ts: None,
        }
    }

    pub fn pk(&self) -> &str {
        &self.pk
    }
    pub fn sk(&self) -> &str {
        &self.sk
    }
    pub fn to_key_item<I: From<Item>>(&self) -> I {
        let h =
            std::collections::HashMap::from([("PK", self.pk.as_str()), ("SK", self.sk.as_str())]);
        serde_dynamo::to_item(h).expect("Should never fail")
    }
    pub fn to_bare_item<I: From<Item>>(&self) -> I {
        serde_dynamo::to_item(self).expect("Should never fail")
    }
    pub fn set_expiration(&mut self, expiration_ts: Option<u64>) {
        self.expiration_ts = expiration_ts;
    }
    pub fn get_expiration(&self) -> Option<u64> {
        self.expiration_ts
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonoItem<T> {
    #[serde(flatten)]
    core: MonoItemCore,
    #[serde(flatten)]
    inner: T,
}

impl<T> MonoItem<T> {
    pub fn new(core: MonoItemCore, inner: T) -> Self {
        MonoItem { core, inner }
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
    pub fn set_expiration(&mut self, expiration_ts: Option<u64>) {
        self.core.expiration_ts = expiration_ts;
    }
    pub fn get_expiration(&self) -> Option<u64> {
        self.core.expiration_ts
    }
}
impl<T: Serialize> MonoItem<T> {
    pub fn to_item<I: From<Item>>(&self) -> I {
        serde_dynamo::to_item(self).expect("Should never fail")
    }
}

impl<T: DeserializeOwned> MonoItem<T> {
    pub fn from_item<I: Into<Item>>(item: I) -> Self {
        serde_dynamo::from_item(item).expect("Should never fail or DB schema corrupted")
    }
}

pub trait ToMonoItemCore {
    fn to_monoitemcore(&self) -> MonoItemCore;
}
impl<'a, T: ToMonoItemCore> From<&'a T> for MonoItem<&'a T> {
    fn from(value: &'a T) -> Self {
        MonoItem::new(<T as ToMonoItemCore>::to_monoitemcore(&value), value)
    }
}

pub trait ItemUpdater {
    fn is_empty(&self) -> bool;
    fn insert_updates(self, builder: UpdateItemFluentBuilder) -> UpdateItemFluentBuilder;
}
