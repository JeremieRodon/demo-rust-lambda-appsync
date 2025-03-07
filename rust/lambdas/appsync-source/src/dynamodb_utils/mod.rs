mod generic;

use std::collections::HashMap;

use aws_sdk_dynamodb::types::{
    builders::{DeleteRequestBuilder, UpdateBuilder},
    AttributeValue, ConditionCheck, ReturnValue, TransactWriteItem, Update, WriteRequest,
};
use generic::{
    dynamodb_batch_write, dynamodb_delete_item, dynamodb_perform_query, dynamodb_perform_scan,
    table_name, DynamoDBItem, DynamoItem, PK, SK, TYPE,
};
use lambda_commons_utils::{log, tokio};
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_attribute_value, to_attribute_value};
use shared_types::common::Uuid;

use crate::{dynamodb, GameState, GameStatus, Player, Team};

impl GameStatus {
    const PK_SK_TYPE: &'static str = "GAME_STATUS";
    const PROPERTY_NAME: &'static str = "game_status";
}
impl DynamoDBItem for GameStatus {
    type Id = ();
    fn get_key(&self) -> DynamoItem {
        Self::get_key_from_id(())
    }
    fn get_key_from_id(_id: Self::Id) -> DynamoItem {
        HashMap::from([
            (
                PK.to_owned(),
                AttributeValue::S(Self::PK_SK_TYPE.to_owned()),
            ),
            (
                SK.to_owned(),
                AttributeValue::S(Self::PK_SK_TYPE.to_owned()),
            ),
        ])
    }
    fn get_type() -> &'static str {
        Self::PK_SK_TYPE
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

    // Create the iterator of BatchWriteRequest that will DELETE every player CLICKS and LATENCY
    let batch_write_requests = players
        .into_iter()
        .flat_map(|player| {
            let id = player.id;
            // Create the two BatchWriteRequest
            [
                WriteRequest::builder()
                    .delete_request(
                        DeleteRequestBuilder::default()
                            .set_key(Some(PlayerClicks::get_key_from_id(id)))
                            .build()
                            .expect("key is set"),
                    )
                    .build(),
                WriteRequest::builder()
                    .delete_request(
                        DeleteRequestBuilder::default()
                            .set_key(Some(PlayerLatency::get_key_from_id(id)))
                            .build()
                            .expect("key is set"),
                    )
                    .build(),
            ]
        })
        .collect::<Vec<_>>();
    dynamodb_batch_write(batch_write_requests).await
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

// We destructure the Player object into 3 pieces
// The goal is to have "Clicks" and "LatencyReports" in different items.
// Before implementing this, I had regular errors during the game because
// it happens (quite often) that a `click` request and a `reportLatency` request
// arrive at the same time and, as they both use a transaction to perform their update,
// they would conflict and one of them was dropped.

// By destructuring, we increase complexity. But in exchange we have "Clicks" and "LatencyReports" in different items,
// we means their transactions cannot conflict anymore.

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PlayerClicks {
    pub id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clicks: Option<i64>,
}
impl PlayerClicks {
    const SK_TYPE: &'static str = "2PLAYER_CLICKS";
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PlayerLatency {
    pub id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_latency: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_latency_clicks: Option<i64>,
}
impl PlayerLatency {
    const SK_TYPE: &'static str = "3PLAYER_LATENCY";
}

macro_rules! impl_dbitem {
    ($struct:ident) => {
        impl DynamoDBItem for $struct {
            type Id = Uuid;

            fn get_key_from_id(id: Self::Id) -> DynamoItem {
                HashMap::from([
                    (PK.to_owned(), AttributeValue::S(Player::pk_from_uuid(id))),
                    (SK.to_owned(), AttributeValue::S(Self::SK_TYPE.to_owned())),
                ])
            }

            fn get_key(&self) -> DynamoItem {
                Self::get_key_from_id(self.id)
            }

            fn get_type() -> &'static str {
                Self::SK_TYPE
            }
        }
    };
}

impl_dbitem!(Player);
impl_dbitem!(PlayerClicks);
impl_dbitem!(PlayerLatency);

impl Player {
    const SK_TYPE: &'static str = "1PLAYER_META";
    fn pk_from_uuid(id: Uuid) -> String {
        format!("PLAYER#{}", id)
    }
    fn restructure_clicks(&mut self, clicks: PlayerClicks) {
        assert_eq!(self.id, clicks.id);
        let PlayerClicks { clicks, .. } = clicks;
        self.clicks = clicks;
    }
    fn restructure_latency(&mut self, latency: PlayerLatency) {
        assert_eq!(self.id, latency.id);
        let PlayerLatency {
            avg_latency,
            avg_latency_clicks,
            ..
        } = latency;
        self.avg_latency = avg_latency;
        self.avg_latency_clicks = avg_latency_clicks;
    }
    fn restructure(&mut self, clicks: PlayerClicks, latency: PlayerLatency) {
        self.restructure_clicks(clicks);
        self.restructure_latency(latency);
    }
}

pub async fn dynamodb_put_new_player(new_player: &Player) -> Result<(), aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_put_new_player - new_player={new_player:?}");

    // No need to destructure or anything: a new player always has clicks/avg_latency/avg_latency_clicks to none
    // Even if it did not, we don't really care...
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
                        .set_key(Some(GameStatus::get_key_from_id(())))
                        .condition_expression(format!(
                            "attribute_exists({PK}) AND {} = :game_status",
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
        .set_key(Some(Player::get_key_from_id(player_id)))
        .update_expression("SET #name = :name")
        .expression_attribute_names("#name", "name")
        .expression_attribute_values(":name", to_attribute_value(new_name).unwrap())
        .condition_expression(format!("attribute_exists({PK})"))
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
        .set_key(Some(PlayerLatency::get_key_from_id(player_id)))
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
            .condition_expression(format!(
                "attribute_exists({PK}) AND #avg_latency = :old_avg_latency \
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
            "attribute_exists({PK}) AND attribute_not_exists(#avg_latency) \
            AND attribute_not_exists(#avg_latency_clicks)"
        )),
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

    dynamodb_get_player(player_id)
        .await
        .map(|op| op.expect("player deleted at an incredible timing that I don't want to manage"))
}
pub async fn dynamodb_get_player(
    player_id: Uuid,
) -> Result<Option<Player>, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_get_player - player_id={player_id}");

    let query = dynamodb()
        .query()
        .table_name(table_name())
        .key_condition_expression(format!("{PK} = :player_id"))
        .expression_attribute_values(":player_id", AttributeValue::S(player_id.to_string()))
        // This is to ensure that 1PLAYER_META is the last of the list
        .scan_index_forward(false);
    let mut items = dynamodb_perform_query(query).await?;

    // The 1PLAYER_META object is guaranteed to be the last of the list by DynamoDB
    // Therefore it is the first to be pop
    let player = if let Some(item) = items.pop() {
        let mut player = Player::from_item(item);
        while let Some(item) = items.pop() {
            if PlayerClicks::is_item(&item) {
                let clicks = PlayerClicks::from_item(item);
                player.restructure_clicks(clicks);
            } else if PlayerLatency::is_item(&item) {
                let latency = PlayerLatency::from_item(item);
                player.restructure_latency(latency);
            }
        }
        Some(player)
    } else {
        None
    };

    Ok(player)
}
pub async fn dynamodb_delete_player(player_id: Uuid) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_delete_player - player_id={player_id}");

    let req_delete_player = tokio::spawn(dynamodb_delete_item(Player::get_key_from_id(player_id)));
    let req_delete_player_clicks = tokio::spawn(dynamodb_delete_item(
        PlayerClicks::get_key_from_id(player_id),
    ));
    let req_delete_player_latency = tokio::spawn(dynamodb_delete_item(
        PlayerLatency::get_key_from_id(player_id),
    ));

    let mut player = Player::from_item(req_delete_player.await.unwrap()?);
    let clicks = PlayerClicks::from_item(req_delete_player_clicks.await.unwrap()?);
    let latency = PlayerLatency::from_item(req_delete_player_latency.await.unwrap()?);
    player.restructure(clicks, latency);

    Ok(player)
}

pub async fn dynamodb_player_click(player_id: Uuid) -> Result<Player, aws_sdk_dynamodb::Error> {
    log::debug!("ENTER dynamodb_player_click - player_id={player_id}");

    dynamodb_update_if_game_started(
        UpdateBuilder::default()
            .table_name(table_name())
            .set_key(Some(PlayerClicks::get_key_from_id(player_id)))
            .update_expression("SET clicks = if_not_exists(clicks, :zero) + :one")
            .condition_expression(format!("attribute_exists({PK})"))
            .expression_attribute_values(":zero", AttributeValue::N("0".to_owned()))
            .expression_attribute_values(":one", AttributeValue::N("1".to_owned()))
            .build()
            .expect("table_name, key and update_expression are set"),
    )
    .await?;

    dynamodb_get_player(player_id)
        .await
        .map(|op| op.expect("player deleted at an incredible timing that I don't want to manage"))
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

    let mut players: HashMap<Uuid, Player> = HashMap::new();
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
        if PlayerClicks::is_item(&item) {
            let clicks = PlayerClicks::from_item(item);
            if let Some(player) = players.get_mut(&clicks.id) {
                player.restructure_clicks(clicks);
            }
        } else if PlayerLatency::is_item(&item) {
            let latency = PlayerLatency::from_item(item);
            if let Some(player) = players.get_mut(&latency.id) {
                player.restructure_latency(latency);
            }
        } else if GameStatus::is_item(&item) {
            game_state.status = GameStatus::from_item(item);
        }
    }
    // Set the players Vec and the state is ready!
    game_state.players = players.into_values().collect();

    Ok(game_state)
}
