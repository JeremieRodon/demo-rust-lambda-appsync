use std::collections::HashSet;

use crate::{
    GameState, GameStatus, Player, Team,
    dynamodb_helpers::{
        dynamodb_delete_player, dynamodb_put_new_player, dynamodb_query_game_state,
        dynamodb_query_teams_player_count, dynamodb_reset_game, dynamodb_set_game_status,
        dynamodb_update_player_name,
    },
};
use lambda_appsync::{AppSyncError, ID, appsync_operation};

fn player_not_found() -> AppSyncError {
    AppSyncError::new("PlayerNotFound", "Player does not exist")
}
fn from_dynamo_error(e: aws_sdk_dynamodb::Error) -> AppSyncError {
    let meta = aws_sdk_dynamodb::error::ProvideErrorMetadata::meta(&e);
    AppSyncError::new(
        meta.code().unwrap_or("Unknown"),
        meta.message().unwrap_or_default(),
    )
}

// impl crate::Operation {
//     pub async fn query_game_state() -> Result<GameState, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::query_game_state().await;
//         }
//         Ok(dynamodb_query_game_state()
//             .await
//             .map_err(from_dynamo_error)?)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(query(gameState))]
pub async fn game_state() -> Result<GameState, AppSyncError> {
    Ok(dynamodb_query_game_state()
        .await
        .map_err(from_dynamo_error)?)
}

macro_rules! game_status_mut {
    ($mut_name:ident, $status:path ) => {
        #[appsync_operation(mutation($mut_name))]
        pub async fn f() -> Result<GameStatus, AppSyncError> {
            dynamodb_set_game_status($status)
                .await
                .map_err(from_dynamo_error)?;
            Ok($status)
        }
    };
}

game_status_mut!(startGame, GameStatus::Started);
game_status_mut!(stopGame, GameStatus::Stopped);

// impl crate::Operation {
//     pub async fn mutation_reset_game() -> Result<GameStatus, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::mutation_reset_game().await;
//         }
//         dynamodb_reset_game().await.map_err(from_dynamo_error)?;
//         Ok(GameStatus::Reset)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(mutation(resetGame))]
pub async fn reset_game() -> Result<GameStatus, AppSyncError> {
    dynamodb_reset_game().await.map_err(from_dynamo_error)?;
    Ok(GameStatus::Reset)
}

// impl crate::Operation {
//     pub async fn mutation_register_new_player(
//         name: String,
//         secret: String,
//     ) -> Result<Player, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::mutation_register_new_player(
//                 name, secret,
//             )
//             .await;
//         }
//         let mut teams_player_count = dynamodb_query_teams_player_count()
//             .await
//             .map_err(from_dynamo_error)?;
//         let team = if teams_player_count.len() < Team::COUNT {
//             // If all teams are not yet used, choose one of the unused
//             let mut all_teams = HashSet::from(Team::all());
//             while let Some((team, _)) = teams_player_count.pop() {
//                 all_teams.remove(&team);
//             }
//             all_teams
//                 .into_iter()
//                 .next()
//                 .expect("we ensured teams_player_count had less element than all_teams")
//         } else {
//             // Else chose the one with less players
//             teams_player_count.sort_by_key(|o| o.1);
//             teams_player_count[0].0
//         };
//         let id = ID::new();
//         let new_player = Player {
//             id,
//             name,
//             team,
//             clicks: None,
//             avg_latency: None,
//             avg_latency_clicks: None,
//         };
//         dynamodb_put_new_player(&new_player, secret)
//             .await
//             .map_err(from_dynamo_error)?;
//         Ok(new_player)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(mutation(registerNewPlayer))]
pub async fn register_new_player(name: String, secret: String) -> Result<Player, AppSyncError> {
    let mut teams_player_count = dynamodb_query_teams_player_count()
        .await
        .map_err(from_dynamo_error)?;
    let team = if teams_player_count.len() < Team::COUNT {
        // If all teams are not yet used, choose one of the unused
        let mut all_teams = HashSet::from(Team::all());
        while let Some((team, _)) = teams_player_count.pop() {
            all_teams.remove(&team);
        }
        all_teams
            .into_iter()
            .next()
            .expect("we ensured teams_player_count had less element than all_teams")
    } else {
        // Else chose the one with less players
        teams_player_count.sort_by_key(|o| o.1);
        teams_player_count[0].0
    };
    let id = ID::new();
    let new_player = Player {
        id,
        name,
        team,
        clicks: None,
        avg_latency: None,
        avg_latency_clicks: None,
    };
    dynamodb_put_new_player(&new_player, secret)
        .await
        .map_err(from_dynamo_error)?;
    Ok(new_player)
}

// impl crate::Operation {
//     pub async fn mutation_update_player_name(
//         player_id: ID,
//         new_name: String,
//         secret: String,
//     ) -> Result<Player, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::mutation_update_player_name(
//                 player_id, new_name, secret,
//             )
//             .await;
//         }
//         Ok(dynamodb_update_player_name(player_id, new_name, secret)
//             .await
//             .map_err(from_dynamo_error)?)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(mutation(updatePlayerName))]
pub async fn update_player_name(
    player_id: ID,
    new_name: String,
    secret: String,
) -> Result<Player, AppSyncError> {
    Ok(dynamodb_update_player_name(player_id, new_name, secret)
        .await
        .map_err(from_dynamo_error)?)
}

// impl crate::Operation {
//     pub async fn mutation_remove_player(player_id: ID) -> Result<Player, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::mutation_remove_player(
//                 player_id,
//             )
//             .await;
//         }
//         Ok(dynamodb_delete_player(player_id)
//             .await
//             .map_err(from_dynamo_error)?
//             .ok_or_else(player_not_found)?)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(mutation(removePlayer))]
pub async fn remove_player(player_id: ID) -> Result<Player, AppSyncError> {
    Ok(dynamodb_delete_player(player_id)
        .await
        .map_err(from_dynamo_error)?
        .ok_or_else(player_not_found)?)
}
