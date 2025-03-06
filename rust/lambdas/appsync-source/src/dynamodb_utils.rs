use std::collections::HashMap;

use aws_sdk_dynamodb::types::{
    builders::{PutRequestBuilder, UpdateBuilder},
    AttributeValue, ConditionCheck, ReturnValue, TransactWriteItem, Update, WriteRequest,
};
use lambda_commons_utils::{log, tokio};
use serde::{de::DeserializeOwned, Serialize};
use serde_dynamo::{from_attribute_value, to_attribute_value};
use shared_types::common::Uuid;

use crate::{dynamodb, GameState, GameStatus, Player, Team};

trait DynamoDBItem: Serialize + DeserializeOwned {
    fn get_key(&self) -> String;
    fn get_type() -> &'static str;
    fn to_item(&self) -> HashMap<String, aws_sdk_dynamodb::types::AttributeValue> {
        let mut item = self.to_item_core();
        let inner: HashMap<String, aws_sdk_dynamodb::types::AttributeValue> =
            serde_dynamo::to_item(self).expect("valid schema");
        item.extend(inner.into_iter());
        item
    }
    fn to_item_core(&self) -> HashMap<String, aws_sdk_dynamodb::types::AttributeValue> {
        HashMap::from([
            ("PK".to_owned(), to_attribute_value(self.get_key()).unwrap()),
            (
                "_TYPE".to_owned(),
                to_attribute_value(Self::get_type()).unwrap(),
            ),
        ])
    }
    fn from_item(item: HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> Self {
        serde_dynamo::from_item(item).expect("valid schema")
    }
    fn is_item(item: &HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> bool {
        item.get("_TYPE")
            .is_some_and(|t| t.as_s().expect("invalid schema") == Self::get_type())
    }
}

fn table_name() -> String {
    let table_name = std::env::var("BACKEND_TABLE_NAME")
        .expect("Mandatory environment variable `BACKEND_TABLE_NAME` is not set");
    log::debug!("BACKEND_TABLE_NAME={table_name}");
    table_name
}
impl GameStatus {
    const PK: &'static str = "GAME_STATUS";
    const PROPERTY_NAME: &'static str = "game_status";
}
impl DynamoDBItem for GameStatus {
    fn get_key(&self) -> String {
        Self::PK.to_owned()
    }
    fn get_type() -> &'static str {
        Self::PK
    }
    fn to_item(&self) -> HashMap<String, aws_sdk_dynamodb::types::AttributeValue> {
        let mut item = self.to_item_core();
        item.insert(
            Self::PROPERTY_NAME.to_owned(),
            to_attribute_value(self).unwrap(),
        );
        item
    }
    fn from_item(mut item: HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> Self {
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

    // Create the iteraotor of BatchWriteRequest that will PUT each player in a stripped_from_counter version
    let mut batch_write_requests = players
        .into_iter()
        .map(|mut player| {
            // Strip player of their counters
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
                tokio::spawn(async move {
                    log::debug!(
                        "dynamodb_reset_game::BATCH - Sending BatchWriteItem for chunk #{index}..."
                    );
                    let result = dynamodb()
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
pub async fn dynamodb_set_game_status(status: GameStatus) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_set_game_status - status={status:?}");
    // Can only set GameStatus in some order
    let current_status = status.valid_from_status();

    dynamodb()
        .put_item()
        .table_name(table_name())
        .set_item(Some(status.to_item()))
        .condition_expression(format!(
            "attribute_not_exists(PK) OR {} = :game_status",
            GameStatus::PROPERTY_NAME
        ))
        .expression_attribute_values(":game_status", to_attribute_value(current_status).unwrap())
        .return_values(ReturnValue::None)
        .send()
        .await?;
    Ok(())
}

impl Player {
    fn key_from_uuid(id: Uuid) -> String {
        format!("PLAYER#{}", id)
    }
}
impl DynamoDBItem for Player {
    fn get_key(&self) -> String {
        Self::key_from_uuid(self.id)
    }
    fn get_type() -> &'static str {
        "PLAYER"
    }
}
pub async fn dynamodb_put_new_player(new_player: &Player) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_put_new_player - new_player={new_player:?}");
    dynamodb()
        .put_item()
        .table_name(table_name())
        .set_item(Some(new_player.to_item()))
        .condition_expression("attribute_not_exists(PK)")
        .return_values(ReturnValue::None)
        .send()
        .await?;
    Ok(())
}
pub async fn dynamodb_update_if_game_started(
    transcat_update: Update,
) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_update_if_game_started");
    dynamodb()
        .transact_write_items()
        .transact_items(TransactWriteItem::builder().update(transcat_update).build())
        .transact_items(
            TransactWriteItem::builder()
                .condition_check(
                    ConditionCheck::builder()
                        .table_name(table_name())
                        .key("PK", to_attribute_value(GameStatus::PK).unwrap())
                        .condition_expression(format!(
                            "attribute_exists(PK) AND {} = :game_status",
                            GameStatus::PROPERTY_NAME
                        ))
                        .expression_attribute_values(
                            ":game_status",
                            to_attribute_value(GameStatus::Started).unwrap(),
                        )
                        .build()
                        .expect("table_name, key and update_expression are set"),
                )
                .build(),
        )
        .send()
        .await?;
    Ok(())
}
pub async fn dynamodb_update_player_name(
    player_id: Uuid,
    new_name: String,
) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_update_player_name - player_id={player_id} new_name={new_name}");
    let updated_item = dynamodb()
        .update_item()
        .table_name(table_name())
        .key(
            "PK",
            to_attribute_value(Player::key_from_uuid(player_id)).unwrap(),
        )
        .update_expression("SET #name = :name")
        .expression_attribute_names("#name", "name")
        .expression_attribute_values(":name", to_attribute_value(new_name).unwrap())
        .condition_expression("attribute_exists(PK)")
        .return_values(ReturnValue::AllNew)
        .send()
        .await?
        .attributes
        .expect("asked for them");
    Ok(Player::from_item(updated_item))
}
pub async fn dynamodb_update_player_latency_stats(
    player_id: Uuid,
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

    let update = UpdateBuilder::default()
        .table_name(table_name())
        .key(
            "PK",
            to_attribute_value(Player::key_from_uuid(player_id)).unwrap(),
        )
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
        );

    let update = match (old_avg_latency, old_avg_latency_clicks) {
        (Some(old_avg_latency), Some(old_avg_latency_clicks)) => update
            .condition_expression(
                "attribute_exists(PK) AND #avg_latency = :old_avg_latency \
            AND #avg_latency_clicks = :old_avg_latency_clicks",
            )
            .expression_attribute_values(
                ":old_avg_latency",
                to_attribute_value(old_avg_latency).unwrap(),
            )
            .expression_attribute_values(
                ":old_avg_latency_clicks",
                to_attribute_value(old_avg_latency_clicks).unwrap(),
            ),
        (None, None) => update.condition_expression(
            "attribute_exists(PK) AND attribute_not_exists(#avg_latency) \
            AND attribute_not_exists(#avg_latency_clicks)",
        ),
        _ => unreachable!(
            "Functionnal error, old_avg_latency and old_avg_latency_clicks \
            can only be both None or both Some"
        ),
    };

    dynamodb_update_if_game_started(
        update
            .build()
            .expect("table_name, key and update_expression are set"),
    )
    .await?;

    dynamodb_get_player(player_id).await
}
pub async fn dynamodb_get_player(player_id: Uuid) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_get_player - player_id={player_id}");
    Ok(Player::from_item(
        dynamodb()
            .get_item()
            .table_name(table_name())
            .key(
                "PK",
                to_attribute_value(Player::key_from_uuid(player_id)).unwrap(),
            )
            .consistent_read(true)
            .send()
            .await?
            .item
            .expect("player deleted between the click transac and get_item"),
    ))
}
pub async fn dynamodb_delete_player(player_id: Uuid) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_delete_player - player_id={player_id}");
    let removed_item = dynamodb()
        .delete_item()
        .table_name(table_name())
        .key(
            "PK",
            to_attribute_value(Player::key_from_uuid(player_id)).unwrap(),
        )
        .condition_expression("attribute_exists(PK)")
        .return_values(ReturnValue::AllOld)
        .send()
        .await?
        .attributes
        .expect("asked for them");
    Ok(Player::from_item(removed_item))
}

pub async fn dynamodb_player_click(player_id: Uuid) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_player_click - player_id={player_id}");

    dynamodb_update_if_game_started(
        UpdateBuilder::default()
            .table_name(table_name())
            .key(
                "PK",
                to_attribute_value(Player::key_from_uuid(player_id)).unwrap(),
            )
            .update_expression("SET clicks = if_not_exists(clicks, :zero) + :one")
            .condition_expression("attribute_exists(PK)")
            .expression_attribute_values(":zero", AttributeValue::N("0".to_owned()))
            .expression_attribute_values(":one", AttributeValue::N("1".to_owned()))
            .build()
            .expect("table_name, key and update_expression are set"),
    )
    .await?;

    dynamodb_get_player(player_id).await
}

pub async fn dynamodb_query_teams_player_count(
) -> Result<Vec<(Team, usize)>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_query_teams_player_count");

    let scan_req_builder = dynamodb()
        .scan()
        .table_name(table_name())
        .projection_expression("PK,#type,team")
        .expression_attribute_names("#type", "_TYPE");

    let scan_res = scan_req_builder.clone().send().await?;
    let mut items = scan_res.items.unwrap_or_default();
    let mut lek = scan_res.last_evaluated_key;
    while lek.is_some() {
        let scan_res = scan_req_builder
            .clone()
            .set_exclusive_start_key(lek)
            .send()
            .await?;
        lek = scan_res.last_evaluated_key;
        items.extend(scan_res.items.unwrap_or_default());
    }

    let teams = items.into_iter().filter_map(|mut item| {
        if item
            .remove("_TYPE")
            .is_some_and(|t| t.as_s().expect("invalid schema") == Player::get_type())
        {
            Some(
                from_attribute_value::<_, Team>(item.remove("team").expect("invalid schema"))
                    .expect("invalid schema"),
            )
        } else {
            None
        }
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
    let scan_res = scan_req_builder.clone().send().await?;
    let mut items = scan_res.items.unwrap_or_default();
    let mut lek = scan_res.last_evaluated_key;
    while lek.is_some() {
        let scan_res = scan_req_builder
            .clone()
            .set_exclusive_start_key(lek)
            .send()
            .await?;
        lek = scan_res.last_evaluated_key;
        items.extend(scan_res.items.unwrap_or_default());
    }

    let mut game_state = GameState {
        status: GameStatus::Reset,
        players: vec![],
    };

    for item in items {
        if Player::is_item(&item) {
            let player = Player::from_item(item);
            game_state.players.push(player);
        } else if GameStatus::is_item(&item) {
            game_state.status = GameStatus::from_item(item);
        }
    }

    Ok(game_state)
}
