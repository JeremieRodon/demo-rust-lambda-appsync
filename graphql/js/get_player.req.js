import { util } from "@aws-appsync/utils";

/**
 * Sends a request to get an item with id `ctx.args.id`
 * @param {import('@aws-appsync/utils').Context} ctx the context
 * @returns {import('@aws-appsync/utils').DynamoDBGetItemRequest} the request
 */
export function request(ctx) {
  // Verify the gamestatus
  const game_status = ctx.prev.result;
  if (!game_status || game_status.game_status != "STARTED") {
    util.error("Game is not started", "InvalidGameStatus");
  }
  return {
    operation: "GetItem",
    key: util.dynamodb.toMapValues({ PK: `PLAYER#${ctx.args.player_id}` }),
  };
}

/**
 * Returns the fetched DynamoDB item
 * @param {import('@aws-appsync/utils').Context} ctx the context
 * @returns {*} the DynamoDB item
 */
export function response(ctx) {
  return ctx.result;
}
