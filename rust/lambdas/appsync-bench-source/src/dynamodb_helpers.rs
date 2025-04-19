use std::collections::HashMap;

use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use dynamodb_utils::{DynamoDBItem, DynamoItem, PK, table_name};

use lambda_appsync::{ID, log};
use serde_dynamo::to_attribute_value;

use crate::{GameStatus, Player, dynamodb};

impl GameStatus {
    /// The partition key (PK) value used to store the game status in DynamoDB
    const PK_TYPE: &'static str = "GAME_STATUS";
    /// The attribute name storing the actual game status value
    const PROPERTY_NAME: &'static str = "game_status";
}

/// DynamoDB storage implementation for [GameStatus]
impl DynamoDBItem for GameStatus {
    type Id = ();

    fn get_key(&self) -> DynamoItem {
        Self::get_key_from_id(())
    }

    fn get_key_from_id(_id: Self::Id) -> DynamoItem {
        // GameStatus is a singleton, so we use a fixed PK
        HashMap::from([(PK.to_owned(), AttributeValue::S(Self::PK_TYPE.to_owned()))])
    }

    fn get_type() -> &'static str {
        Self::PK_TYPE
    }

    fn to_item(&self) -> DynamoItem {
        let mut item = self.to_item_core();
        // Store the enum value under the PROPERTY_NAME attribute
        item.insert(
            Self::PROPERTY_NAME.to_owned(),
            to_attribute_value(self).unwrap(),
        );
        item
    }

    fn from_item(mut item: DynamoItem) -> Self {
        // Extract and deserialize the enum value
        item.remove(Self::PROPERTY_NAME)
            .map(|a| serde_dynamo::from_attribute_value(a).expect("valid schema"))
            .expect("valid schema")
    }
}

/// Retrieves the current [GameStatus] from DynamoDB
///
/// # Returns
/// Returns  [Ok(None)] if no game status is set yet
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
    /// The partition key (PK) prefix used for player items
    const PK_TYPE: &'static str = "PLAYER";

    /// Generates the full partition key for a player from their ID
    fn pk_from_uuid(id: ID) -> String {
        format!("{}#{}", Self::PK_TYPE, id)
    }
}

/// DynamoDB storage implementation for Player
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

/// Retrieves a [Player] from DynamoDB by their ID
///
/// # Returns
/// Returns [Ok(None)] if the player does not exist
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

/// Increments a player's click counter atomically, after verifying their secret
///
/// If the clicks attribute doesn't exist yet, it will be initialized to 1
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
        // Verify the player exists and the secret matches
        .condition_expression(format!("attribute_exists({PK}) AND secret = :secret"))
        .return_values(ReturnValue::AllNew)
        .send()
        .await?
        .attributes
        .map(Player::from_item)
        .expect("asked for them"))
}

/// Updates a player's latency statistics, using optimistic locking to prevent concurrent updates
///
/// The old values are used as update conditions to ensure no concurrent update happened. They should either both be None
/// (for first update) or both be Some.
/// Note that concurrent updates should never happen because the frontend is set to send a report per second,
/// which is plently enough to finish an update before the following one.
pub async fn dynamodb_update_player_latency_stats(
    player_id: ID,
    secret: String,
    old_avg_latency: Option<f64>,
    old_avg_latency_clicks: Option<i32>,
    new_avg_latency: f64,
    new_avg_latency_clicks: i32,
) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!(
        "ENTER dynamodb_update_player_latency_stats - \
        player_id={player_id} \
        old_avg_latency={old_avg_latency:?} old_avg_latency_clicks={old_avg_latency_clicks:?} \
        new_avg_latency={new_avg_latency} new_avg_latency_clicks={new_avg_latency_clicks}"
    );

    // Start building the update operation with the new values
    let update = dynamodb()
        .update_item()
        .table_name(table_name())
        .set_key(Some(Player::get_key_from_id(player_id)))
        .update_expression(
            "SET #avg_latency = :new_avg_latency, #avg_latency_clicks = :new_avg_latency_clicks",
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

    // Add optimistic locking condition based on old values
    let update = match (old_avg_latency, old_avg_latency_clicks) {
        // If we had previous values, ensure they haven't changed
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
        // For first update, ensure attributes don't exist yet
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
