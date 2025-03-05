use std::collections::HashMap;

use aws_sdk_dynamodb::{
    types::{AttributeValue, ReturnValue},
    Error as DynamoErr,
};
use lambda_commons_utils::log;
use serde::Serialize;
pub use shared_types::dynamodb::{ItemUpdater, MonoItem, MonoItemCore, ToMonoItemCore};

pub(crate) async fn put_item_dynamo<T: std::fmt::Debug + Serialize + ToMonoItemCore>(
    item: &T,
) -> Result<(), DynamoErr> {
    log::debug!("ENTER put_item_dynamo");
    log::debug!("item={item:?}");
    let table_name = std::env::var("BACKEND_TABLE_NAME")
        .expect("Mandatory environment variable `BACKEND_TABLE_NAME` is not set");
    log::debug!("BACKEND_TABLE_NAME={table_name}");
    crate::dynamodb()
        .put_item()
        .table_name(table_name)
        .set_item(Some(MonoItem::from(item).to_item()))
        .condition_expression("attribute_not_exists(PK)")
        .send()
        .await?;
    Ok(())
}

pub(crate) async fn update_item_dynamo<T: ItemUpdater>(
    key: HashMap<String, AttributeValue>,
    updater: T,
) -> Result<HashMap<String, AttributeValue>, DynamoErr> {
    log::debug!("ENTER update_item_dynamodb");
    log::debug!("key={key:?}");
    let table_name = std::env::var("BACKEND_TABLE_NAME")
        .expect("Mandatory environment variable `BACKEND_TABLE_NAME` is not set");
    log::debug!("BACKEND_TABLE_NAME={table_name}");

    let builder = crate::dynamodb()
        .update_item()
        .table_name(&table_name)
        .set_key(Some(key))
        .condition_expression("attribute_exists(PK)")
        .return_values(ReturnValue::AllNew);
    let builder = updater.insert_updates(builder);
    let result = builder.send().await?;
    Ok(result
        .attributes
        .expect("condition expression ensure its present"))
}

pub(crate) async fn remove_item_dynamo(
    key: HashMap<String, AttributeValue>,
) -> Result<HashMap<String, AttributeValue>, DynamoErr> {
    log::debug!("ENTER remove_item_dynamo");
    log::debug!("key={key:?}");
    let table_name = std::env::var("BACKEND_TABLE_NAME")
        .expect("Mandatory environment variable `BACKEND_TABLE_NAME` is not set");
    log::debug!("BACKEND_TABLE_NAME={table_name}");
    let result = crate::dynamodb()
        .delete_item()
        .table_name(&table_name)
        .set_key(Some(key))
        .condition_expression("attribute_exists(PK)")
        .return_values(ReturnValue::AllOld)
        .send()
        .await?;

    Ok(result
        .attributes
        .expect("condition expression ensure its present"))
}
