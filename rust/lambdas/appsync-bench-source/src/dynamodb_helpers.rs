use std::collections::HashMap;

use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use dynamodb_utils::{DynamoDBItem, DynamoItem, PK, table_name};

use lambda_appsync::{ID, log};
use serde_dynamo::to_attribute_value;

use crate::{GameStatus, Player, dynamodb};

impl GameStatus {
    const PK_TYPE: &'static str = "GAME_STATUS";
    const PROPERTY_NAME: &'static str = "game_status";
}
impl DynamoDBItem for GameStatus {
    type Id = ();
    fn get_key(&self) -> DynamoItem {
        Self::get_key_from_id(())
    }
    fn get_key_from_id(_id: Self::Id) -> DynamoItem {
        HashMap::from([(PK.to_owned(), AttributeValue::S(Self::PK_TYPE.to_owned()))])
    }
    fn get_type() -> &'static str {
        Self::PK_TYPE
    }
    fn to_item(&self) -> DynamoItem {
        let mut item = self.to_item_core();
        item.insert(
            Self::PROPERTY_NAME.to_owned(),
            to_attribute_value(self).unwrap(),
        );
        item
    }
    fn from_item(mut item: DynamoItem) -> Self {
        item.remove(Self::PROPERTY_NAME)
            .map(|a| serde_dynamo::from_attribute_value(a).expect("valid schema"))
            .expect("valid schema")
    }
}

pub async fn dynamodb_get_game_status() -> Result<Option<GameStatus>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_get_game_status");

    Ok(dynamodb()
        .get_item()
        .table_name(table_name())
        .set_key(Some(GameStatus::get_key_from_id(())))
        .send()
        .await?
        .item
        .map(GameStatus::from_item))
}

impl Player {
    const PK_TYPE: &'static str = "PLAYER";
    fn pk_from_uuid(id: ID) -> String {
        format!("{}#{}", Self::PK_TYPE, id)
    }
}

impl DynamoDBItem for Player {
    type Id = ID;

    fn get_key(&self) -> DynamoItem {
        Self::get_key_from_id(self.id)
    }
    fn get_key_from_id(id: Self::Id) -> DynamoItem {
        HashMap::from([(PK.to_owned(), AttributeValue::S(Self::pk_from_uuid(id)))])
    }
    fn get_type() -> &'static str {
        Self::PK_TYPE
    }
}

pub async fn dynamodb_get_player(player_id: ID) -> Result<Option<Player>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_get_player - player_id={player_id}");

    Ok(dynamodb()
        .get_item()
        .table_name(table_name())
        .set_key(Some(Player::get_key_from_id(player_id)))
        .send()
        .await?
        .item
        .map(Player::from_item))
}

pub async fn dynamodb_update_player_click(
    player_id: ID,
    secret: String,
) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_player_click - player_id={player_id}");
    Ok(dynamodb()
        .update_item()
        .table_name(table_name())
        .set_key(Some(Player::get_key_from_id(player_id)))
        .update_expression("SET #clicks = if_not_exists(#clicks, :zero) + :one")
        .expression_attribute_names("#clicks", "clicks")
        .expression_attribute_values(":zero", AttributeValue::N("0".to_owned()))
        .expression_attribute_values(":one", AttributeValue::N("1".to_owned()))
        .expression_attribute_values(":secret", AttributeValue::S(secret))
        .condition_expression(format!("attribute_exists({PK}) AND secret = :secret"))
        .return_values(ReturnValue::AllNew)
        .send()
        .await?
        .attributes
        .map(Player::from_item)
        .expect("asked for them"))
}

pub async fn dynamodb_update_player_latency_stats(
    player_id: ID,
    secret: String,
    old_avg_latency: Option<f64>,
    old_avg_latency_clicks: Option<i64>,
    new_avg_latency: f64,
    new_avg_latency_clicks: i64,
) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!(
        "ENTER dynamodb_update_player_latency_stats - \
        player_id={player_id} \
        old_avg_latency={old_avg_latency:?} old_avg_latency_clicks={old_avg_latency_clicks:?} \
        new_avg_latency={new_avg_latency} new_avg_latency_clicks={new_avg_latency_clicks}
        "
    );

    let update = dynamodb()
        .update_item()
        .table_name(table_name())
        .set_key(Some(Player::get_key_from_id(player_id)))
        .update_expression(
            "SET #avg_latency = :new_avg_latency, \
        #avg_latency_clicks = :new_avg_latency_clicks",
        )
        .expression_attribute_names("#avg_latency", "avg_latency")
        .expression_attribute_names("#avg_latency_clicks", "avg_latency_clicks")
        .expression_attribute_values(
            ":new_avg_latency",
            to_attribute_value(new_avg_latency).unwrap(),
        )
        .expression_attribute_values(
            ":new_avg_latency_clicks",
            to_attribute_value(new_avg_latency_clicks).unwrap(),
        )
        .expression_attribute_values(":secret", AttributeValue::S(secret))
        .condition_expression(format!("attribute_exists({PK}) AND secret = :secret"));

    let update = match (old_avg_latency, old_avg_latency_clicks) {
        (Some(old_avg_latency), Some(old_avg_latency_clicks)) => update
            .condition_expression(format!(
                "attribute_exists({PK}) AND secret = :secret \
                AND #avg_latency = :old_avg_latency \
                AND #avg_latency_clicks = :old_avg_latency_clicks"
            ))
            .expression_attribute_values(
                ":old_avg_latency",
                to_attribute_value(old_avg_latency).unwrap(),
            )
            .expression_attribute_values(
                ":old_avg_latency_clicks",
                to_attribute_value(old_avg_latency_clicks).unwrap(),
            ),
        (None, None) => update.condition_expression(format!(
            "attribute_exists({PK}) AND secret = :secret \
            AND attribute_not_exists(#avg_latency) \
            AND attribute_not_exists(#avg_latency_clicks)"
        )),
        _ => unreachable!(
            "Functionnal error, old_avg_latency and old_avg_latency_clicks \
            can only be both None or both Some"
        ),
    };
    Ok(update
        .return_values(ReturnValue::AllNew)
        .send()
        .await?
        .attributes
        .map(Player::from_item)
        .expect("asked for them"))
}
