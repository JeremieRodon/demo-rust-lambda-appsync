use std::collections::HashSet;

use crate::{
    dynamodb_helpers::{
        dynamodb_delete_player, dynamodb_put_new_player, dynamodb_query_game_state,
        dynamodb_query_teams_player_count, dynamodb_reset_game, dynamodb_set_game_status,
        dynamodb_update_player_name,
    },
    GameState, GameStatus, Player, Team,
};
use appsync_utils::{AppSyncError, ID};

fn player_not_found() -> AppSyncError {
    AppSyncError::new("PlayerNotFound", "Player does not exist")
}

impl crate::Operation {
    pub async fn query_game_state() -> Result<GameState, AppSyncError> {
        // This is just a marker to ensure an error is thrown if the user did not chose
        // the correct signature for the function. Should be optimized away by the compiler.
        if false {
            return <crate::Operation as crate::DefautOperations>::query_game_state().await;
        }
        Ok(dynamodb_query_game_state().await?)
    }
}

macro_rules! game_status_mut {
    ($mut_name:ident, $status:path ) => {
        impl crate::Operation {
            pub async fn $mut_name() -> Result<GameStatus, AppSyncError> {
                // This is just a marker to ensure an error is thrown if the user did not chose
                // the correct signature for the function. Should be optimized away by the compiler.
                if false {
                    return <crate::Operation as crate::DefautOperations>::$mut_name().await;
                }
                dynamodb_set_game_status($status).await?;
                Ok($status)
            }
        }
    };
}

game_status_mut!(mutation_start_game, GameStatus::Started);
game_status_mut!(mutation_stop_game, GameStatus::Stopped);

impl crate::Operation {
    pub async fn mutation_reset_game() -> Result<GameStatus, AppSyncError> {
        // This is just a marker to ensure an error is thrown if the user did not chose
        // the correct signature for the function. Should be optimized away by the compiler.
        if false {
            return <crate::Operation as crate::DefautOperations>::mutation_reset_game().await;
        }
        dynamodb_reset_game().await?;
        Ok(GameStatus::Reset)
    }
}

impl crate::Operation {
    pub async fn mutation_register_new_player(name: String) -> Result<Player, AppSyncError> {
        // This is just a marker to ensure an error is thrown if the user did not chose
        // the correct signature for the function. Should be optimized away by the compiler.
        if false {
            return <crate::Operation as crate::DefautOperations>::mutation_register_new_player(
                name,
            )
            .await;
        }
        let mut teams_player_count = dynamodb_query_teams_player_count().await?;
        let team = if teams_player_count.len() < Team::TEAM_COUNT {
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
        dynamodb_put_new_player(&new_player).await?;
        Ok(new_player)
    }
}

impl crate::Operation {
    pub async fn mutation_update_player_name(
        player_id: ID,
        new_name: String,
    ) -> Result<Player, AppSyncError> {
        // This is just a marker to ensure an error is thrown if the user did not chose
        // the correct signature for the function. Should be optimized away by the compiler.
        if false {
            return <crate::Operation as crate::DefautOperations>::mutation_update_player_name(
                player_id, new_name,
            )
            .await;
        }
        Ok(dynamodb_update_player_name(player_id, new_name).await?)
    }
}

impl crate::Operation {
    pub async fn mutation_remove_player(player_id: ID) -> Result<Player, AppSyncError> {
        // This is just a marker to ensure an error is thrown if the user did not chose
        // the correct signature for the function. Should be optimized away by the compiler.
        if false {
            return <crate::Operation as crate::DefautOperations>::mutation_remove_player(
                player_id,
            )
            .await;
        }
        Ok(dynamodb_delete_player(player_id)
            .await?
            .ok_or_else(player_not_found)?)
    }
}
