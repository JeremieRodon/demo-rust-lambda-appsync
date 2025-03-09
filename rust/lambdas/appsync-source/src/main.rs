mod dynamodb_helpers;
mod game;
mod operations;

use appsync_utils::{
    arg_from_json, res_to_json, AppSyncError, AppSyncEvent, AppSyncEventInfo, AppSyncResponse, ID,
};
use serde::{Deserialize, Serialize};

use lambda_commons_utils::prelude::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum GameStatus {
    Started,
    Stopped,
    Reset,
}
impl GameStatus {
    pub const COUNT: usize = 3;
    pub fn all() -> [Self; Self::COUNT] {
        [Self::Started, Self::Stopped, Self::Reset]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum Team {
    Rust,
    // Python,
    Js,
    Vtl,
}
impl Team {
    pub const COUNT: usize = 3; //4;
    pub fn all() -> [Self; Self::COUNT] {
        [
            Self::Rust,
            //Self::Python,
            Self::Js,
            Self::Vtl,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: ID,
    pub name: String,
    pub team: Team,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clicks: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_latency: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_latency_clicks: Option<i64>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct LatencyReport {
    clicks: i64,
    avg_latency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    status: GameStatus,
    players: Vec<Player>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryField {
    GameState,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MutationField {
    StartGame,
    StopGame,
    ResetGame,
    RegisterNewPlayer,
    UpdatePlayerName,
    RemovePlayer,
    ClickRust,
    ClickJs,
    ClickVtl,
    ReportLatencyRust,
    ReportLatencyJs,
    ReportLatencyVtl,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionField {
    UpdatedPlayer,
    RemovedPlayer,
    UpdatedGameStatus,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "parentTypeName", content = "fieldName")]
pub enum Operation {
    Query(QueryField),
    Mutation(MutationField),
    Subscription(SubscriptionField),
}

#[allow(dead_code)]
trait DefautOperations {
    async fn query_game_state() -> Result<GameState, AppSyncError> {
        unimplemented!("Query `gameState` is unimplemented")
    }
    async fn mutation_start_game() -> Result<GameStatus, AppSyncError> {
        unimplemented!("Mutation `startGame` is unimplemented")
    }
    async fn mutation_stop_game() -> Result<GameStatus, AppSyncError> {
        unimplemented!("Mutation `stopGame` is unimplemented")
    }
    async fn mutation_reset_game() -> Result<GameStatus, AppSyncError> {
        unimplemented!("Mutation `resetGame` is unimplemented")
    }
    async fn mutation_register_new_player(_name: String) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `registerNewPlayer` is unimplemented")
    }
    async fn mutation_update_player_name(
        _player_id: ID,
        _new_name: String,
    ) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `updatePlayerName` is unimplemented")
    }
    async fn mutation_remove_player(_player_id: ID) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `removePlayer` is unimplemented")
    }
    async fn mutation_click_rust(_player_id: ID) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `clickRust` is unimplemented")
    }
    async fn mutation_click_js(_player_id: ID) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `clickJs` is unimplemented")
    }
    async fn mutation_click_vtl(_player_id: ID) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `clickVtl` is unimplemented")
    }
    async fn mutation_report_latency_rust(
        _player_id: ID,
        _report: LatencyReport,
    ) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `reportLatencyRust` is unimplemented")
    }
    async fn mutation_report_latency_js(
        _player_id: ID,
        _report: LatencyReport,
    ) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `reportLatencyJs` is unimplemented")
    }
    async fn mutation_report_latency_vtl(
        _player_id: ID,
        _report: LatencyReport,
    ) -> Result<Player, AppSyncError> {
        unimplemented!("Mutation `reportLatencyVtl` is unimplemented")
    }
    async fn subscription_updated_player() -> Result<(), AppSyncError> {
        Ok(())
    }
    async fn subscription_removed_player() -> Result<(), AppSyncError> {
        Ok(())
    }
    async fn subscription_updated_game_status() -> Result<(), AppSyncError> {
        Ok(())
    }
}
impl DefautOperations for Operation {}

impl Operation {
    pub async fn execute(self, args: serde_json::Value) -> AppSyncResponse {
        match self._execute(args).await {
            Ok(v) => AppSyncResponse {
                data: Some(v),
                error: None,
            },
            Err(e) => {
                log::error!("{e}");
                AppSyncResponse {
                    data: None,
                    error: Some(e),
                }
            }
        }
    }

    async fn _execute(
        self,
        mut args: serde_json::Value,
    ) -> Result<serde_json::Value, AppSyncError> {
        match self {
            Operation::Query(query_field) => match query_field {
                QueryField::GameState => Operation::query_game_state().await.map(res_to_json),
            },
            Operation::Mutation(mutation_field) => match mutation_field {
                MutationField::StartGame => Operation::mutation_start_game().await.map(res_to_json),
                MutationField::StopGame => Operation::mutation_stop_game().await.map(res_to_json),
                MutationField::ResetGame => Operation::mutation_reset_game().await.map(res_to_json),
                MutationField::RegisterNewPlayer => {
                    Operation::mutation_register_new_player(arg_from_json(&mut args, "name")?)
                        .await
                        .map(res_to_json)
                }
                MutationField::UpdatePlayerName => Operation::mutation_update_player_name(
                    arg_from_json(&mut args, "player_id")?,
                    arg_from_json(&mut args, "new_name")?,
                )
                .await
                .map(res_to_json),
                MutationField::RemovePlayer => {
                    Operation::mutation_remove_player(arg_from_json(&mut args, "player_id")?)
                        .await
                        .map(res_to_json)
                }
                MutationField::ClickRust => {
                    Operation::mutation_click_rust(arg_from_json(&mut args, "player_id")?)
                        .await
                        .map(res_to_json)
                }
                MutationField::ClickJs => {
                    Operation::mutation_click_js(arg_from_json(&mut args, "player_id")?)
                        .await
                        .map(res_to_json)
                }
                MutationField::ClickVtl => {
                    Operation::mutation_click_vtl(arg_from_json(&mut args, "player_id")?)
                        .await
                        .map(res_to_json)
                }
                MutationField::ReportLatencyRust => Operation::mutation_report_latency_rust(
                    arg_from_json(&mut args, "player_id")?,
                    arg_from_json(&mut args, "report")?,
                )
                .await
                .map(res_to_json),
                MutationField::ReportLatencyJs => Operation::mutation_report_latency_js(
                    arg_from_json(&mut args, "player_id")?,
                    arg_from_json(&mut args, "report")?,
                )
                .await
                .map(res_to_json),
                MutationField::ReportLatencyVtl => Operation::mutation_report_latency_vtl(
                    arg_from_json(&mut args, "player_id")?,
                    arg_from_json(&mut args, "report")?,
                )
                .await
                .map(res_to_json),
            },
            Operation::Subscription(subscription_field) => match subscription_field {
                SubscriptionField::UpdatedPlayer => Operation::subscription_updated_player()
                    .await
                    .map(res_to_json),
                SubscriptionField::RemovedPlayer => Operation::subscription_removed_player()
                    .await
                    .map(res_to_json),
                SubscriptionField::UpdatedGameStatus => {
                    Operation::subscription_updated_game_status()
                        .await
                        .map(res_to_json)
                }
            },
        }
    }
}

async fn handler(event: AppSyncEvent<Operation>) -> AppSyncResponse {
    log::info!("event={event:?}");
    let AppSyncEvent {
        identity: _,
        request: _,
        source: _,
        info:
            AppSyncEventInfo {
                operation,
                selection_set_graphql: _,
                selection_set_list: _,
                variables: _,
            },
        args,
    } = event;
    log::info!("operation={operation:?}");
    operation.execute(args).await
}

async fn batch_handler(
    events: Vec<AppSyncEvent<Operation>>,
) -> Result<Vec<AppSyncResponse>, std::convert::Infallible> {
    let handles = events
        .into_iter()
        .map(|e| tokio::spawn(handler(e)))
        .collect::<Vec<_>>();

    let mut results = vec![];
    for h in handles {
        results.push(h.await.unwrap())
    }
    Ok(results)
}

lambda_main!(async batch_handler(Vec<AppSyncEvent<Operation>>)->Vec<AppSyncResponse>, dynamodb = aws_sdk_dynamodb::Client);
