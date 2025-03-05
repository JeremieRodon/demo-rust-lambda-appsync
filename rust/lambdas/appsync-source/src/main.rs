mod appsync_utils;
mod dynamodb_utils;

use appsync_utils::{
    AppSyncError, AppSyncEvent, AppSyncIdentity, AppSyncResponse, MutationField, Operation,
    QueryField,
};

use lambda_commons_utils::prelude::*;

fn invalid_args(arg_name: &str, e: impl core::error::Error) -> AppSyncError {
    AppSyncError::new(
        "InvalidArgs",
        format!("Argument \"{arg_name}\" is not the expected format ({e})"),
    )
}

macro_rules! op_invoke {
    ($args:ident, $f:ident($($arg:literal,)*)) => {
        $f(
            $(
                serde_json::from_value($args.get_mut($arg)
                        .unwrap_or(&mut serde_json::Value::Null)
                        .take(),
                ).map_err(|e| invalid_args($arg, e))?,
            )*
        ).await
        .map(|r| serde_json::to_value(r).expect("cannot fail"))
    };
}

async fn handler(event: AppSyncEvent) -> Result<serde_json::Value, AppSyncError> {
    log::info!("event={event:?}");
    let AppSyncEvent {
        identity,
        request: _,
        source: _,
        info,
        mut args,
    } = event;
    log::info!("operation={info:?}");

    match info {
        Operation::Query(query_field) => match query_field {
            QueryField::GameState => todo!(),
            // QueryField::GuestGroups => op_invoke!(args, guest_groups()),
            // QueryField::GuestGroup => op_invoke!(args, guest_group("id",)),
            // QueryField::TentativeGuestRequests => op_invoke!(args, tentative_guest_requests()),
            // QueryField::Strings => op_invoke!(args, get_strings()),
        },
        Operation::Mutation(mutation_field) => match mutation_field {
            MutationField::StartGame => todo!(),
            MutationField::StopGame => todo!(),
            MutationField::ResetGame => todo!(),
            MutationField::RegisterNewPlayer => todo!(),
            MutationField::UpdatePlayerName => todo!(),
            MutationField::RemovePlayer => todo!(),
            MutationField::Click => todo!(),
            MutationField::ReportLatency => todo!(),
            // MutationField::CreateGuestGroup => {
            //     op_invoke!(args, create_guest_group("name", "guests",))
            // }
        },
        Operation::Subscription(_) => Ok(serde_json::Value::Null),
    }
}

async fn batch_handler(
    events: Vec<AppSyncEvent>,
) -> Result<Vec<AppSyncResponse>, std::convert::Infallible> {
    let handles = events
        .into_iter()
        .map(|e| tokio::spawn(handler(e)))
        .collect::<Vec<_>>();

    let mut results = vec![];
    for h in handles {
        match h.await.unwrap() {
            Ok(v) => results.push(AppSyncResponse {
                data: Some(v),
                error: None,
            }),
            Err(e) => {
                log::error!("{e}");
                results.push(AppSyncResponse {
                    data: None,
                    error: Some(e),
                })
            }
        }
    }

    Ok(results)
}

lambda_main!(async batch_handler(Vec<AppSyncEvent>)->Vec<AppSyncResponse>, dynamodb = aws_sdk_dynamodb::Client);
