/**
 * Sends a request to the attached data source
 * @param {import('@aws-appsync/utils').Context} ctx the context
 * @returns {*} the request
 */
export function request(ctx) {
  // The report values
  const { clicks, avg_latency } = ctx.args.report;
  // The player previous values
  const player = ctx.prev.result;
  const old_avg_latency = player.avg_latency;
  const old_avg_latency_clicks = player.avg_latency_clicks;

  const has_previous_values = old_avg_latency_clicks != null;

  const old_total_latency = has_previous_values
    ? old_avg_latency * old_avg_latency_clicks
    : 0.0;
  const new_total_latency = old_total_latency + avg_latency * clicks;

  const new_avg_latency_clicks =
    (has_previous_values ? old_avg_latency_clicks : 0) + clicks;
  const new_avg_latency = new_total_latency / new_avg_latency_clicks;

  const condition = has_previous_values
    ? util.transform.toDynamoDBConditionExpression({
        PK: { exist: true },
        avg_latency: { eq: old_avg_latency },
        avg_latency_clicks: { eq: old_avg_latency_clicks },
      })
    : util.transform.toDynamoDBConditionExpression({
        PK: { exist: true },
        avg_latency: { exist: false },
        avg_latency_clicks: { exist: false },
      });

  return {
    operation: "UpdateItem",
    key: util.dynamodb.toMapValues({ PK: `PLAYER#${ctx.args.player_id}` }),
    update: {
      expression:
        "SET #avg_latency = :new_avg_latency, #avg_latency_clicks = :new_avg_latency_clicks",
      expressionNames: {
        "#avg_latency": "avg_latency",
        "#avg_latency_clicks": "avg_latency_clicks",
      },
      expressionValues: {
        ":new_avg_latency": util.dynamodb.toDynamoDB(new_avg_latency),
        ":new_avg_latency_clicks": util.dynamodb.toDynamoDB(
          new_avg_latency_clicks
        ),
      },
    },
    condition,
  };
}

/**
 * Returns the resolver result
 * @param {import('@aws-appsync/utils').Context} ctx the context
 * @returns {*} the result
 */
export function response(ctx) {
  // Update with response logic
  return ctx.result;
}
