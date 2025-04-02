use std::collections::HashSet;

use crate::{
    GameStatus, Player, Team,
    dynamodb_helpers::{
        dynamodb_delete_player, dynamodb_get_game_status, dynamodb_put_new_player,
        dynamodb_query_players, dynamodb_query_teams_player_count, dynamodb_reset_game,
        dynamodb_set_game_status, dynamodb_update_player_name,
    },
};
use lambda_appsync::{AppsyncError, ID, appsync_operation};

fn player_not_found() -> AppsyncError {
    AppsyncError::new("PlayerNotFound", "Player does not exist")
}

// impl crate::Operation {
//     pub async fn query_players() -> Result<Vec<Player>, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::query_players().await;
//         }
//         Ok(dynamodb_query_players().await?)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(query(players))]
pub async fn players() -> Result<Vec<Player>, AppsyncError> {
    Ok(dynamodb_query_players().await?)
}
// impl crate::Operation {
//     pub async fn query_game_status() -> Result<GameStatus, AppSyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::query_game_status().await;
//         }
//         Ok(dynamodb_get_game_status().await?)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(query(gameStatus))]
pub async fn game_status() -> Result<GameStatus, AppsyncError> {
    Ok(dynamodb_get_game_status()
        .await?
        .unwrap_or_else(|| GameStatus::Reset))
}

// This is a declarative macro that helps reduce boilerplate code for game status mutation operations.
// It generates a function for each game status mutation (like startGame and stopGame) that follows
// the same pattern but with different GameStatus values.
macro_rules! game_status_mut {
    // The macro takes two parameters:
    // $mut_name: The identifier for the mutation name (like startGame)
    // $status: The path to the GameStatus variant to set (like GameStatus::Started)
    ($mut_name:ident, $status:path ) => {
        // The macro generates an async function annotated with appsync_operation
        // indicating this is a GraphQL mutation handler
        #[appsync_operation(mutation($mut_name))]
        pub async fn _discarded() -> Result<GameStatus, AppsyncError> {
            // Update the game status in DynamoDB to the new status
            dynamodb_set_game_status($status).await?;
            // Return the new status on success
            Ok($status)
        }
    };
}

// Generate two mutation handlers:
// - startGame: Sets game status to Started
// - stopGame: Sets game status to Stopped
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
pub async fn reset_game() -> Result<GameStatus, AppsyncError> {
    dynamodb_reset_game().await?;
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
pub async fn register_new_player(name: String, secret: String) -> Result<Player, AppsyncError> {
    // Query DynamoDB to get the current count of players in each team
    let mut teams_player_count = dynamodb_query_teams_player_count().await?;

    // Choose which team to assign this player to
    let team = if teams_player_count.len() < Team::COUNT {
        // If all teams are not yet used, choose one of the unused teams
        let mut all_teams = HashSet::from(Team::all());
        while let Some((team, _)) = teams_player_count.pop() {
            all_teams.remove(&team);
        }
        // Get the first unused team
        all_teams
            .into_iter()
            .next()
            .expect("we ensured teams_player_count had less element than all_teams")
    } else {
        // If all teams are used, choose the one with the fewest players
        teams_player_count.sort_by_key(|o| o.1);
        teams_player_count[0].0
    };

    // Generate a new unique ID for this player
    let id = ID::new();

    // Create the new player record
    let new_player = Player {
        id,
        name,
        team,
        clicks: None,
        avg_latency: None,
        avg_latency_clicks: None,
    };

    // Save the new player to DynamoDB
    dynamodb_put_new_player(&new_player, secret).await?;

    // Return the newly created player
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
) -> Result<Player, AppsyncError> {
    Ok(dynamodb_update_player_name(player_id, new_name, secret).await?)
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
pub async fn remove_player(player_id: ID) -> Result<Player, AppsyncError> {
    Ok(dynamodb_delete_player(player_id)
        .await?
        .ok_or_else(player_not_found)?)
}
