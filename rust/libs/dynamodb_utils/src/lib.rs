//! Utility module for working with DynamoDB tables that use a simple partition key ("PK") schema.
//!
//! This module provides a trait and helper functions for implementing a typed interface over a
//! DynamoDB table. The table schema assumes:
//! - A single partition key named "PK"
//! - Type discrimination using "_TYPE" attribute
//! - Values serializable/deserializable via serde

use std::collections::HashMap;

use aws_sdk_dynamodb::{
    operation::scan::builders::ScanFluentBuilder,
    types::{AttributeValue, ReturnValue, WriteRequest},
};

use serde::{Serialize, de::DeserializeOwned};

/// Name of the partition key attribute
pub static PK: &'static str = "PK";

/// Name of the type discriminator attribute
pub static TYPE: &'static str = "_TYPE";

/// Type alias for a DynamoDB item represented as a HashMap
pub type DynamoItem = HashMap<String, aws_sdk_dynamodb::types::AttributeValue>;

/// Trait for types that can be stored in DynamoDB with type information
pub trait DynamoDBItem: Serialize + DeserializeOwned {
    /// The type used to generate the item's partition key
    type Id;

    /// Generates a DynamoDB key from an ID value
    fn get_key_from_id(id: Self::Id) -> DynamoItem;

    /// Gets the DynamoDB key for this item
    fn get_key(&self) -> DynamoItem;

    /// Gets the type discriminator string for this type
    fn get_type() -> &'static str;

    /// Converts this item into a DynamoDB item with type information
    fn to_item(&self) -> DynamoItem {
        let mut item = self.to_item_core();
        let inner: DynamoItem = serde_dynamo::to_item(self).expect("valid schema");
        item.extend(inner.into_iter());
        item
    }

    /// Internal helper to generate the base DynamoDB item with key and type
    fn to_item_core(&self) -> DynamoItem {
        let mut k = self.get_key();
        k.insert(
            TYPE.to_owned(),
            AttributeValue::S(Self::get_type().to_owned()),
        );
        k
    }

    /// Creates an instance from a DynamoDB item
    fn from_item(item: DynamoItem) -> Self {
        serde_dynamo::from_item(item).expect("valid schema")
    }

    /// Checks if a DynamoDB item matches this type
    fn is_item(item: &DynamoItem) -> bool {
        item.get(TYPE)
            .is_some_and(|t| t.as_s().expect("valid schema") == Self::get_type())
    }
}

/// Gets the DynamoDB table name from the `BACKEND_TABLE_NAME` environment variable
pub fn table_name() -> String {
    let table_name = std::env::var("BACKEND_TABLE_NAME")
        .expect("Mandatory environment variable `BACKEND_TABLE_NAME` is not set");
    log::debug!("BACKEND_TABLE_NAME={table_name}");
    table_name
}

/// Writes items to DynamoDB in batches, with retry logic for unprocessed items
pub async fn dynamodb_batch_write(
    client: aws_sdk_dynamodb::Client,
    mut batch_write_requests: Vec<WriteRequest>,
) -> Result<(), aws_sdk_dynamodb::Error> {
    // Process the Batch(es) in massively parallel fashion
    // Because Rust.
    log::debug!(
        "dynamodb_reset_game::BATCH - putting {} items...",
        batch_write_requests.len()
    );
    let mut retry = 0;
    while batch_write_requests.len() > 0 && retry < 5 {
        retry += 1;
        log::debug!("dynamodb_reset_game::BATCH - Try #{retry}/5");
        let handles = batch_write_requests
            .chunks(25)
            .enumerate()
            .map(|(index, chunk)| {
                let chunk = chunk.to_vec();
                let cclient = client.clone();
                tokio::spawn(async move {
                    log::debug!(
                        "dynamodb_reset_game::BATCH - Sending BatchWriteItem for chunk #{index}..."
                    );
                    let result = cclient
                        .batch_write_item()
                        .set_request_items(Some([(table_name(), chunk)].into()))
                        .send()
                        .await;
                    log::debug!(
                        "dynamodb_reset_game::BATCH - BatchWriteItem finished for chunk #{index}"
                    );
                    result
                })
            })
            .collect::<Vec<_>>();
        let mut unprocess_vec = Vec::default();

        for h in handles {
            let batch_output = h.await.unwrap()?;
            if let Some(unproccessed) = batch_output.unprocessed_items {
                if unproccessed.len() > 0 {
                    unprocess_vec.extend(unproccessed.into_iter().map(|e| e.1).flatten());
                }
            }
        }

        batch_write_requests = unprocess_vec;

        log::debug!(
            "dynamodb_reset_game::BATCH - {} items were unprocessed",
            batch_write_requests.len()
        );
    }

    Ok(())
}

/// Deletes an item from DynamoDB and returns its previous value if it existed
pub async fn dynamodb_delete_item(
    client: aws_sdk_dynamodb::Client,
    key: DynamoItem,
) -> Result<Option<DynamoItem>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_delete_item - key={key:?}");
    Ok(client
        .delete_item()
        .table_name(table_name())
        .set_key(Some(key))
        .condition_expression(format!("attribute_exists({PK})"))
        .return_values(ReturnValue::AllOld)
        .send()
        .await
        .map(|delete_output| delete_output.attributes)?)
}

/// Performs a complete table scan using the provided DynamoDB Scan builder, handling pagination automatically
pub async fn dynamodb_perform_scan(
    builder: ScanFluentBuilder,
) -> Result<Vec<DynamoItem>, aws_sdk_dynamodb::Error> {
    let res = builder.clone().send().await?;
    let mut items = res.items.unwrap_or_default();
    let mut lek = res.last_evaluated_key;
    while lek.is_some() {
        let res = builder.clone().set_exclusive_start_key(lek).send().await?;
        lek = res.last_evaluated_key;
        items.extend(res.items.unwrap_or_default());
    }
    Ok(items)
}
