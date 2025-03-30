use crate::{
    GameStatus, LatencyReport, Player,
    dynamodb_helpers::{
        dynamodb_get_game_status, dynamodb_get_player, dynamodb_update_player_click,
        dynamodb_update_player_latency_stats,
    },
};

use lambda_appsync::{AppsyncError, ID, appsync_operation};

fn player_not_found() -> AppsyncError {
    AppsyncError::new("PlayerNotFound", "Player does not exist")
}
fn invalid_game_status() -> AppsyncError {
    AppsyncError::new("InvalidGameStatus", "Game is not started")
}

// impl crate::Operation {
//     pub async fn mutation_click_rust(
//         player_id: ID,
//         secret: String,
//     ) -> Result<Player, AppsyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::mutation_click_rust(
//                 player_id, secret,
//             )
//             .await;
//         }
//         let game_status = dynamodb_get_game_status()
//             .await
//             .map_err(from_dynamo_error)?
//             .ok_or_else(invalid_game_status)?;
//         if game_status != GameStatus::Started {
//             return Err(invalid_game_status());
//         }
//         Ok(dynamodb_update_player_click(player_id, secret)
//             .await
//             .map_err(from_dynamo_error)?)
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(mutation(clickRust))]
pub async fn click(player_id: ID, secret: String) -> Result<Player, AppsyncError> {
    // Retrieve the current game status
    let game_status = dynamodb_get_game_status()
        .await?
        .ok_or_else(invalid_game_status)?;
    // If the game is not "Started", then we return an error
    if game_status != GameStatus::Started {
        return Err(invalid_game_status());
    }
    // Else we increment the click_counter of the player
    Ok(dynamodb_update_player_click(player_id, secret).await?)
}

// impl crate::Operation {
//     pub async fn mutation_report_latency_rust(
//         player_id: ID,
//         report: LatencyReport,
//         secret: String,
//     ) -> Result<Player, AppsyncError> {
//         // This is just a marker to ensure an error is thrown if the user did not chose
//         // the correct signature for the function. Should be optimized away by the compiler.
//         if false {
//             return <crate::Operation as crate::DefautOperations>::mutation_report_latency_rust(
//                 player_id, report, secret,
//             )
//             .await;
//         }
//         let player_req = lambda_appsync::tokio::spawn(dynamodb_get_player(player_id));
//         let game_status = dynamodb_get_game_status()
//             .await
//             .map_err(from_dynamo_error)?
//             .ok_or_else(invalid_game_status)?;
//         if game_status != GameStatus::Started {
//             return Err(invalid_game_status());
//         }
//         // Retrieve the current player
//         let player = player_req
//             .await
//             .unwrap()
//             .map_err(from_dynamo_error)?
//             .ok_or_else(player_not_found)?;
//         let LatencyReport {
//             clicks,
//             avg_latency,
//         } = report;
//         let old_avg_latency = player.avg_latency;
//         let old_avg_latency_clicks = player.avg_latency_clicks;
//         let old_total_latency = match (old_avg_latency, old_avg_latency_clicks) {
//             (Some(old_avg_latency), Some(old_avg_latency_clicks)) => {
//                 old_avg_latency * (old_avg_latency_clicks as f64)
//             }
//             (None, None) => 0f64,
//             _ => unreachable!(
//                 "Functionnal error, old_avg_latency and old_avg_latency_clicks \
//         can only be both None or both Some"
//             ),
//         };
//         let new_total_latency = old_total_latency + avg_latency * (clicks as f64);
//         let new_avg_latency_clicks = old_avg_latency_clicks.unwrap_or_default() + clicks;

//         let new_avg_latency = new_total_latency / (new_avg_latency_clicks as f64);
//         if new_avg_latency.is_finite() {
//             Ok(dynamodb_update_player_latency_stats(
//                 player_id,
//                 secret,
//                 old_avg_latency,
//                 old_avg_latency_clicks,
//                 new_avg_latency,
//                 new_avg_latency_clicks,
//             )
//             .await
//             .map_err(from_dynamo_error)?)
//         } else {
//             Ok(player)
//         }
//     }
// }
// This macro replace the whole function by the code commented above
#[appsync_operation(mutation(reportLatencyRust))]
pub async fn report_latency(
    player_id: ID,
    report: LatencyReport,
    secret: String,
) -> Result<Player, AppsyncError> {
    // Kick off an async request to get the player data first, so it can run in parallel
    // with the game status check that follows
    let player_req = lambda_appsync::tokio::spawn(dynamodb_get_player(player_id));

    // Verify the game is currently in progress
    let game_status = dynamodb_get_game_status()
        .await?
        .ok_or_else(invalid_game_status)?;
    if game_status != GameStatus::Started {
        return Err(invalid_game_status());
    }

    // Wait for and retrieve the player data we requested earlier
    let player = player_req.await.unwrap()?.ok_or_else(player_not_found)?;

    // Extract the values from the latency report:
    // - clicks: how many clicks were made during this reporting period
    // - avg_latency: the average latency (in ms) measured for those clicks
    let LatencyReport {
        clicks,
        avg_latency,
    } = report;

    // Get the player's current statistics:
    // - old_avg_latency: their current average latency across all clicks
    // - old_avg_latency_clicks: how many clicks that average is based on
    let old_avg_latency = player.avg_latency;
    let old_avg_latency_clicks = player.avg_latency_clicks;

    // Calculate the total cumulative latency from all previous clicks.
    // If this is the player's first report (both values None), start at 0.
    // Otherwise multiply their current average by number of clicks to get total.
    let old_total_latency = match (old_avg_latency, old_avg_latency_clicks) {
        (Some(old_avg_latency), Some(old_avg_latency_clicks)) => {
            old_avg_latency * (old_avg_latency_clicks as f64)
        }
        (None, None) => 0f64,
        _ => unreachable!(
            "Functionnal error, old_avg_latency and old_avg_latency_clicks \
        can only be both None or both Some"
        ),
    };

    // Add the new latency total to the cumulative total:
    // new latency total = old latency total + this report's average * number of clicks in this report
    let new_total_latency = old_total_latency + avg_latency * (clicks as f64);

    // Update the total click count by adding new clicks to the old total (or to 0 if first report)
    let new_avg_latency_clicks = old_avg_latency_clicks.unwrap_or_default() + clicks;

    // Calculate the new overall average:
    // total latency across all clicks / total number of clicks
    let new_avg_latency = new_total_latency / (new_avg_latency_clicks as f64);

    // Only update the stats in the database if we got a valid new average latency
    // (protects against division by zero or other invalid calculations)
    if new_avg_latency.is_finite() {
        // Call the update functions, with the old and the new values so it can perform a conditional update
        Ok(dynamodb_update_player_latency_stats(
            player_id,
            secret,
            old_avg_latency,
            old_avg_latency_clicks,
            new_avg_latency,
            new_avg_latency_clicks,
        )
        .await?)
    } else {
        // If the calculation gave invalid results, return the player unchanged
        Ok(player)
    }
}
