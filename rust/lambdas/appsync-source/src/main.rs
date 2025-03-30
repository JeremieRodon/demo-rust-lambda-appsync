mod dynamodb_helpers;
mod game;
mod operations;

// use lambda_appsync::{
//     arg_from_json, aws_config, env_logger, lambda_runtime, log, res_to_json,
//     serde::{Deserialize, Serialize},
//     serde_json, AppSyncError, AppSyncEvent, AppSyncEventInfo, AppSyncResponse, ID,
// };
// use std::sync::OnceLock;

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum GameStatus {
//     #[serde(rename = "STARTED")]
//     Started,
//     #[serde(rename = "STOPPED")]
//     Stopped,
//     #[serde(rename = "RESET")]
//     Reset,
// }
// impl GameStatus {
//     pub const COUNT: usize = 3;
//     pub fn all() -> [Self; Self::COUNT] {
//         [Self::Started, Self::Stopped, Self::Reset]
//     }
// }

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum Team {
//     #[serde(rename = "RUST")]
//     Rust,
//     #[serde(rename = "PYTHON")]
//     Python,
//     #[serde(rename = "JS")]
//     Js,
//     #[serde(rename = "VTL")]
//     Vtl,
// }
// impl Team {
//     pub const COUNT: usize = 4;
//     pub fn all() -> [Self; Self::COUNT] {
//         [Self::Rust, Self::Python, Self::Js, Self::Vtl]
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Player {
//     pub id: ID,
//     pub name: String,
//     pub team: Team,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     pub clicks: Option<i64>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     pub avg_latency: Option<f64>,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     pub avg_latency_clicks: Option<i64>,
// }

// #[derive(Debug, Clone, Copy, Deserialize)]
// pub struct LatencyReport {
//     pub clicks: i64,
//     pub avg_latency: f64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct GameState {
//     pub status: GameStatus,
//     pub players: Vec<Player>,
// }

// #[derive(Debug, Clone, Copy, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum QueryField {
//     GameState,
// }
// #[derive(Debug, Clone, Copy, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum MutationField {
//     StartGame,
//     StopGame,
//     ResetGame,
//     RegisterNewPlayer,
//     UpdatePlayerName,
//     RemovePlayer,
//     ClickRust,
//     ClickJs,
//     ClickVtl,
//     ClickPython,
//     ReportLatencyRust,
//     ReportLatencyJs,
//     ReportLatencyVtl,
//     ReportLatencyPython,
// }
// #[derive(Debug, Clone, Copy, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum SubscriptionField {
//     UpdatedPlayer,
//     RemovedPlayer,
//     UpdatedGameStatus,
// }

// #[derive(Debug, Clone, Copy, Deserialize)]
// #[serde(tag = "parentTypeName", content = "fieldName")]
// pub enum Operation {
//     Query(QueryField),
//     Mutation(MutationField),
//     Subscription(SubscriptionField),
// }

// #[allow(dead_code)]
// trait DefautOperations {
//     async fn query_game_state() -> Result<GameState, AppSyncError> {
//         unimplemented!("Query `gameState` is unimplemented")
//     }
//     async fn mutation_start_game() -> Result<GameStatus, AppSyncError> {
//         unimplemented!("Mutation `startGame` is unimplemented")
//     }
//     async fn mutation_stop_game() -> Result<GameStatus, AppSyncError> {
//         unimplemented!("Mutation `stopGame` is unimplemented")
//     }
//     async fn mutation_reset_game() -> Result<GameStatus, AppSyncError> {
//         unimplemented!("Mutation `resetGame` is unimplemented")
//     }
//     async fn mutation_remove_player(_player_id: ID) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `removePlayer` is unimplemented")
//     }
//     async fn mutation_register_new_player(
//         _name: String,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `registerNewPlayer` is unimplemented")
//     }
//     async fn mutation_update_player_name(
//         _player_id: ID,
//         _new_name: String,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `updatePlayerName` is unimplemented")
//     }
//     async fn mutation_click_rust(_player_id: ID, _secret: String) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `clickRust` is unimplemented")
//     }
//     async fn mutation_click_js(_player_id: ID, _secret: String) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `clickJs` is unimplemented")
//     }
//     async fn mutation_click_vtl(_player_id: ID, _secret: String) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `clickVtl` is unimplemented")
//     }
//     async fn mutation_click_python(
//         _player_id: ID,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `clickPython` is unimplemented")
//     }
//     async fn mutation_report_latency_rust(
//         _player_id: ID,
//         _report: LatencyReport,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `reportLatencyRust` is unimplemented")
//     }
//     async fn mutation_report_latency_js(
//         _player_id: ID,
//         _report: LatencyReport,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `reportLatencyJs` is unimplemented")
//     }
//     async fn mutation_report_latency_vtl(
//         _player_id: ID,
//         _report: LatencyReport,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `reportLatencyVtl` is unimplemented")
//     }
//     async fn mutation_report_latency_python(
//         _player_id: ID,
//         _report: LatencyReport,
//         _secret: String,
//     ) -> Result<Player, AppSyncError> {
//         unimplemented!("Mutation `reportLatencyPython` is unimplemented")
//     }
//     async fn subscription_updated_player() -> Result<(), AppSyncError> {
//         Ok(())
//     }
//     async fn subscription_removed_player() -> Result<(), AppSyncError> {
//         Ok(())
//     }
//     async fn subscription_updated_game_status() -> Result<(), AppSyncError> {
//         Ok(())
//     }
// }
// impl DefautOperations for Operation {}

// impl Operation {
//     pub async fn execute(self, args: serde_json::Value) -> AppSyncResponse {
//         match self._execute(args).await {
//             Ok(v) => AppSyncResponse {
//                 data: Some(v),
//                 error: None,
//             },
//             Err(e) => {
//                 log::error!("{e}");
//                 AppSyncResponse {
//                     data: None,
//                     error: Some(e),
//                 }
//             }
//         }
//     }

//     async fn _execute(
//         self,
//         mut args: serde_json::Value,
//     ) -> Result<serde_json::Value, AppSyncError> {
//         match self {
//             Operation::Query(query_field) => match query_field {
//                 QueryField::GameState => Operation::query_game_state().await.map(res_to_json),
//             },
//             Operation::Mutation(mutation_field) => match mutation_field {
//                 MutationField::StartGame => Operation::mutation_start_game().await.map(res_to_json),
//                 MutationField::StopGame => Operation::mutation_stop_game().await.map(res_to_json),
//                 MutationField::ResetGame => Operation::mutation_reset_game().await.map(res_to_json),
//                 MutationField::RegisterNewPlayer => Operation::mutation_register_new_player(
//                     arg_from_json(&mut args, "name")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::UpdatePlayerName => Operation::mutation_update_player_name(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "new_name")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::RemovePlayer => {
//                     Operation::mutation_remove_player(arg_from_json(&mut args, "player_id")?)
//                         .await
//                         .map(res_to_json)
//                 }
//                 MutationField::ClickRust => Operation::mutation_click_rust(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ClickJs => Operation::mutation_click_js(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ClickVtl => Operation::mutation_click_vtl(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ClickPython => Operation::mutation_click_python(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ReportLatencyRust => Operation::mutation_report_latency_rust(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "report")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ReportLatencyJs => Operation::mutation_report_latency_js(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "report")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ReportLatencyVtl => Operation::mutation_report_latency_vtl(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "report")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//                 MutationField::ReportLatencyPython => Operation::mutation_report_latency_python(
//                     arg_from_json(&mut args, "player_id")?,
//                     arg_from_json(&mut args, "report")?,
//                     arg_from_json(&mut args, "secret")?,
//                 )
//                 .await
//                 .map(res_to_json),
//             },
//             Operation::Subscription(subscription_field) => match subscription_field {
//                 SubscriptionField::UpdatedPlayer => Operation::subscription_updated_player()
//                     .await
//                     .map(res_to_json),
//                 SubscriptionField::RemovedPlayer => Operation::subscription_removed_player()
//                     .await
//                     .map(res_to_json),
//                 SubscriptionField::UpdatedGameStatus => {
//                     Operation::subscription_updated_game_status()
//                         .await
//                         .map(res_to_json)
//                 }
//             },
//         }
//     }
// }
// async fn appsync_handler(event: AppSyncEvent<Operation>) -> AppSyncResponse {
//     log::info!("event={event:?}");
//     log::info!("operation={:?}", event.info.operation);
//     let AppSyncEvent {
//         identity: _,
//         request: _,
//         source: _,
//         info:
//             AppSyncEventInfo {
//                 operation,
//                 selection_set_graphql: _,
//                 selection_set_list: _,
//                 variables: _,
//             },
//         args,
//     } = event;
//     log::info!("operation={operation:?}");
//     operation.execute(args).await
// }
// async fn appsync_batch_handler(
//     events: Vec<::lambda_appsync::AppSyncEvent<Operation>>,
// ) -> Vec<::lambda_appsync::AppSyncResponse> {
//     let handles = events
//         .into_iter()
//         .map(|e| ::lambda_appsync::tokio::spawn(appsync_handler(e)))
//         .collect::<Vec<_>>();
//     let mut results = vec![];
//     for h in handles {
//         results.push(h.await.unwrap())
//     }
//     results
// }
// async fn function_handler(
//     event: ::lambda_appsync::lambda_runtime::LambdaEvent<::lambda_appsync::serde_json::Value>,
// ) -> Result<Vec<::lambda_appsync::AppSyncResponse>, ::lambda_appsync::lambda_runtime::Error> {
//     log::debug!("{event:?}");
//     log::info!("{}", ::lambda_appsync::serde_json::json!(event.payload));
//     Ok(appsync_batch_handler(serde_json::from_value(event.payload)?).await)
// }

// static AWS_SDK_CONFIG: OnceLock<aws_config::SdkConfig> = OnceLock::new();
// pub fn aws_sdk_config() -> &'static aws_config::SdkConfig {
//     AWS_SDK_CONFIG.get().unwrap()
// }
// pub fn dynamodb() -> aws_sdk_dynamodb::Client {
//     static CLIENT: OnceLock<aws_sdk_dynamodb::Client> = OnceLock::new();
//     CLIENT
//         .get_or_init(|| aws_sdk_dynamodb::Client::new(aws_sdk_config()))
//         .clone()
// }

// use ::lambda_appsync::tokio;
// #[tokio::main]
// async fn main() -> Result<(), lambda_runtime::Error> {
//     env_logger::Builder::from_env(
//         env_logger::Env::default()
//             .default_filter_or("info,tracing::span=warn")
//             .default_write_style_or("never"),
//     )
//     .format_timestamp_micros()
//     .init();
//     AWS_SDK_CONFIG
//         .set(aws_config::load_from_env().await)
//         .unwrap();
//     lambda_runtime::run(lambda_runtime::service_fn(function_handler)).await
// }

// This macro generates all the code commented above
// For the types and operations specific to this AppSync project, it uses the GraphQL schema file as a reference
// for the Lambda handler and integration types, it uses generic (and opiniated) event structs
lambda_appsync::appsync_lambda_main! ("graphql/schema.gql", dynamodb() -> aws_sdk_dynamodb::Client);
