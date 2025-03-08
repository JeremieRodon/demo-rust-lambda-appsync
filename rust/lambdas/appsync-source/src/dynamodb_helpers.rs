use std::collections::HashMap;

use appsync_utils::ID;
use aws_sdk_dynamodb::types::{
    builders::PutRequestBuilder, AttributeValue, ReturnValue, WriteRequest,
};
use dynamodb_utils::{
    dynamodb_batch_write, dynamodb_delete_item, dynamodb_perform_scan, table_name, DynamoDBItem,
    DynamoItem, PK, TYPE,
};
use lambda_commons_utils::log;
use serde_dynamo::{from_attribute_value, to_attribute_value};

use crate::{dynamodb, GameState, GameStatus, Player, Team};

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

pub async fn dynamodb_reset_game() -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_reset_game");
    // Start by changing the state to Reset
    // It serves to verify we are actualy in the correct state pour doing that
    // It also prevents any further usage of the "click" button
    dynamodb_set_game_status(GameStatus::Reset).await?;

    // Note that from this point and until we finish cleaning the players, the game is
    // in a somewhat incorrect state: the status is technically `Reset` but players still have scores.
    // This is just a demo, so we will accept that fact.

    // List players
    let GameState { players, .. } = dynamodb_query_game_state().await?;

    // Create the iterator of BatchWriteRequest that will PUT every players without clicks/latency
    let batch_write_requests = players
        .into_iter()
        .map(|mut player| {
            player.clicks = None;
            player.avg_latency = None;
            player.avg_latency_clicks = None;
            // Create the BatchWriteRequest
            WriteRequest::builder()
                .put_request(
                    PutRequestBuilder::default()
                        .set_item(Some(player.to_item()))
                        .build()
                        .expect("item is set"),
                )
                .build()
        })
        .collect::<Vec<_>>();
    dynamodb_batch_write(dynamodb(), batch_write_requests).await
}

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

pub async fn dynamodb_put_new_player(new_player: &Player) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_put_new_player - new_player={new_player:?}");

    dynamodb()
        .put_item()
        .table_name(table_name())
        .set_item(Some(new_player.to_item()))
        .condition_expression(format!("attribute_not_exists({PK})"))
        .return_values(ReturnValue::None)
        .send()
        .await?;

    Ok(())
}
pub async fn dynamodb_update_player_name(
    player_id: ID,
    new_name: String,
) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_update_player_name - player_id={player_id} new_name={new_name}");

    Ok(dynamodb()
        .update_item()
        .table_name(table_name())
        .set_key(Some(Player::get_key_from_id(player_id)))
        .update_expression("SET #name = :name")
        .expression_attribute_names("#name", "name")
        .expression_attribute_values(":name", to_attribute_value(new_name).unwrap())
        .condition_expression(format!("attribute_exists({PK})"))
        .return_values(ReturnValue::AllNew)
        .send()
        .await?
        .attributes
        .map(Player::from_item)
        .expect("asked for them"))
}

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

pub async fn dynamodb_query_teams_player_count(
) -> Result<Vec<(Team, usize)>, aws_sdk_dynamodb::Error> {
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
        counts
            .entry(team)
            .and_modify(|c| {
                *c += 1;
            })
            .or_insert(1usize);
    }

    Ok(counts
        .into_iter()
        .map(|(team, count)| (team, count))
        .collect())
}

pub async fn dynamodb_query_game_state() -> Result<GameState, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_query_game_state");

    let scan_req_builder = dynamodb().scan().table_name(table_name());
    let items = dynamodb_perform_scan(scan_req_builder).await?;

    let mut game_state = GameState {
        status: GameStatus::Reset,
        players: vec![],
    };

    let mut players: HashMap<ID, Player> = HashMap::new();
    // We do 2-pass on the scan result to ensure we found all the PLAYER_META
    // before processing the PLAYER_CLICKS & PLAYER_LATENCY
    // The alternative is to sort the scan result, but it will be O(n.log(n)) operation
    // whereas the 2-pass is guaranteed to be O(2.n), which is better if there are more than 4 elements.
    let (first_pass, second_pass): (Vec<_>, Vec<_>) =
        items.into_iter().partition(|item| Player::is_item(&item));
    for item in first_pass {
        let player = Player::from_item(item);
        players.insert(player.id, player);
    }
    // Now, we will have every "PLAYER_META" players, we can proceed with PLAYER_CLICKS & PLAYER_LATENCY
    // And also GameStatus
    for item in second_pass {
        if Player::is_item(&item) {
            let player = Player::from_item(item);
            game_state.players.push(player);
        } else if GameStatus::is_item(&item) {
            game_state.status = GameStatus::from_item(item);
        }
    }
    // Set the players Vec and the state is ready!
    game_state.players = players.into_values().collect();

    Ok(game_state)
}
