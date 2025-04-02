use std::collections::HashMap;

use aws_sdk_dynamodb::types::{
    AttributeValue, ReturnValue, WriteRequest, builders::PutRequestBuilder,
};
use dynamodb_utils::{
    DynamoDBItem, DynamoItem, PK, TYPE, dynamodb_batch_write, dynamodb_delete_item,
    dynamodb_perform_scan, table_name,
};
use lambda_appsync::{ID, log};

use serde_dynamo::{from_attribute_value, to_attribute_value};

use crate::{GameStatus, Player, Team, dynamodb};

/// Represents the game status in DynamoDB
/// Contains constants and implementations for DynamoDB storage
impl GameStatus {
    /// Partition key prefix for GameStatus items
    const PK_TYPE: &'static str = "GAME_STATUS";
    /// Name of attribute storing the actual game status value
    const PROPERTY_NAME: &'static str = "game_status";
}

/// DynamoDB storage implementation for [GameStatus]
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

/// Retrieves all [Player] items from DynamoDB as raw [DynamoItem]
/// Used internally by query functions that need access to the full item data
async fn dynamodb_list_player_items() -> Result<Vec<DynamoItem>, aws_sdk_dynamodb::Error> {
    let scan_req_builder = dynamodb()
        .scan()
        .table_name(table_name())
        .filter_expression("#type = :player_type")
        .expression_attribute_names("#type", TYPE)
        .expression_attribute_values(
            ":player_type",
            AttributeValue::S(Player::get_type().to_owned()),
        );

    Ok(dynamodb_perform_scan(scan_req_builder).await?)
}

/// Resets the game state and clears all player scores
///
/// First sets game status to [GameStatus::Reset], then removes all score-related attributes
/// from player records while preserving other player data
pub async fn dynamodb_reset_game() -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_reset_game");
    // Start by changing the state to Reset
    // It serves to verify we are actualy in the correct state pour doing that
    // It also prevents any further usage of the "click" button
    dynamodb_set_game_status(GameStatus::Reset).await?;

    // Note that from this point and until we finish cleaning the players, the game is
    // in a somewhat incorrect state: the status is technically `Reset` but players still have scores.
    // This is just a demo, so we will accept that fact.

    // List players as DynamoItem
    // Because we want to retrieve the `secret` field and put it back with the PutItem
    let player_items = dynamodb_list_player_items().await?;

    // Create the iterator of BatchWriteRequest that will PUT every players without clicks/latency
    let batch_write_requests = player_items
        .into_iter()
        .map(|mut player_item| {
            player_item.remove("clicks");
            player_item.remove("avg_latency");
            player_item.remove("avg_latency_clicks");
            // Create the BatchWriteRequest
            WriteRequest::builder()
                .put_request(
                    PutRequestBuilder::default()
                        .set_item(Some(player_item))
                        .build()
                        .expect("item is set"),
                )
                .build()
        })
        .collect::<Vec<_>>();
    dynamodb_batch_write(dynamodb(), batch_write_requests).await
}

/// Updates the game status in DynamoDB
///
/// Enforces valid state transitions by checking the current status matches
/// what is expected for the requested new status
pub async fn dynamodb_set_game_status(status: GameStatus) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_set_game_status - status={status:?}");
    // Can only set GameStatus in some order
    let current_status = status.valid_from_status();

    dynamodb()
        .put_item()
        .table_name(table_name())
        .set_item(Some(status.to_item()))
        .condition_expression(format!(
            "attribute_not_exists({PK}) OR {} = :game_status",
            GameStatus::PROPERTY_NAME
        ))
        .expression_attribute_values(":game_status", to_attribute_value(current_status).unwrap())
        .return_values(ReturnValue::None)
        .send()
        .await?;
    Ok(())
}

/// DynamoDB storage interface for Player records
impl Player {
    /// Partition key prefix for Player items
    const PK_TYPE: &'static str = "PLAYER";

    /// Generates the partition key for a player ID
    fn pk_from_uuid(id: ID) -> String {
        format!("{}#{}", Self::PK_TYPE, id)
    }
}

/// DynamoDB table interface implementation for Player
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

/// Creates a new player record in DynamoDB
///
/// Adds the provided secret along with the player data for future authentication of the player
pub async fn dynamodb_put_new_player(
    new_player: &Player,
    secret: String,
) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_put_new_player - new_player={new_player:?}");

    let mut player_item = new_player.to_item();
    // Add secret
    player_item.insert("secret".to_owned(), AttributeValue::S(secret));

    dynamodb()
        .put_item()
        .table_name(table_name())
        .set_item(Some(player_item))
        .condition_expression(format!("attribute_not_exists({PK})"))
        .return_values(ReturnValue::None)
        .send()
        .await?;

    Ok(())
}

/// Updates a player's name after verifying their secret
///
/// Returns the updated [Player] record
pub async fn dynamodb_update_player_name(
    player_id: ID,
    new_name: String,
    secret: String,
) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_update_player_name - player_id={player_id} new_name={new_name}");

    Ok(dynamodb()
        .update_item()
        .table_name(table_name())
        .set_key(Some(Player::get_key_from_id(player_id)))
        .update_expression("SET #name = :name")
        .expression_attribute_names("#name", "name")
        .expression_attribute_values(":name", to_attribute_value(new_name).unwrap())
        .expression_attribute_values(":secret", AttributeValue::S(secret))
        .condition_expression(format!("attribute_exists({PK}) AND secret = :secret"))
        .return_values(ReturnValue::AllNew)
        .send()
        .await?
        .attributes
        .map(Player::from_item)
        .expect("asked for them"))
}

/// Deletes a player record from DynamoDB
///
/// Returns the deleted [Player] if it existed
pub async fn dynamodb_delete_player(
    player_id: ID,
) -> Result<Option<Player>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_delete_player - player_id={player_id}");

    Ok(
        dynamodb_delete_item(dynamodb(), Player::get_key_from_id(player_id))
            .await?
            .map(Player::from_item),
    )
}

/// Queries DynamoDB to get a count of players per team
///
/// Returns a vector of ([Team], count) tuples
pub async fn dynamodb_query_teams_player_count()
-> Result<Vec<(Team, usize)>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_query_teams_player_count");

    let scan_req_builder = dynamodb()
        .scan()
        .table_name(table_name())
        .filter_expression("#type = :player_type")
        .projection_expression(format!("{PK},#type,team"))
        .expression_attribute_names("#type", TYPE)
        .expression_attribute_values(
            ":player_type",
            AttributeValue::S(Player::get_type().to_owned()),
        );

    let items = dynamodb_perform_scan(scan_req_builder).await?;

    let teams = items.into_iter().map(|mut item| {
        from_attribute_value::<_, Team>(item.remove("team").expect("invalid schema"))
            .expect("invalid schema")
    });
    let mut counts = HashMap::new();
    for team in teams {
        *counts.entry(team).or_insert(0usize) += 1;
    }

    Ok(counts
        .into_iter()
        .map(|(team, count)| (team, count))
        .collect())
}

/// Retrieves all players from DynamoDB
///
/// Returns a vector of Player objects
pub async fn dynamodb_query_players() -> Result<Vec<Player>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_query_players");

    // List players as DynamoItem
    let player_items = dynamodb_list_player_items().await?;
    // Map to Player objects
    Ok(player_items.into_iter().map(Player::from_item).collect())
}

/// Retrieves the current game status from DynamoDB
///
/// Returns the GameStatus enum value
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
