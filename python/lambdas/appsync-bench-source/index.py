# This Python lambda function is minimaly commented
# Please refer the the Rust version of this lambda, which you may find more readable
import json
import os
from decimal import Decimal

import boto3
dyndbr = boto3.resource('dynamodb')
backend_table = dyndbr.Table(os.environ['BACKEND_TABLE_NAME'])

import logging
log_level = logging.INFO
# create logger
logger = logging.getLogger('common')
logger.setLevel(log_level)
logger.propagate = False
# create console handler
ch = logging.StreamHandler()
ch.setLevel(log_level)
# create formatter
formatter = logging.Formatter('[%(asctime)s][%(threadName)s]%(levelname)s - %(message)s')
# add formatter to ch
ch.setFormatter(formatter)
# add ch to logger
logger.addHandler(ch)


# Custom exception class for AppSync errors, similar to Rust's AppSyncError
class AppSyncError(Exception):
    def __init__(self, error_type, error_message):
        self.error_type = error_type
        self.error_message = error_message

# Get the current game status from DynamoDB using partition key 'GAME_STATUS'
def get_game_status():
    return (backend_table.get_item(Key={'PK':'GAME_STATUS'})
            .get('Item', {})
            .get('game_status'))

# Fetch a player record from DynamoDB using player ID
def get_player(player_id):
    player = (backend_table
            .get_item(Key={'PK':f'PLAYER#{player_id}'})
            .get('Item'))
    if player is None:
        return None
    del player['PK']
    return player

# Atomically increment a player's click counter in DynamoDB after verifying their secret
def update_click(player_id, secret):
    player = backend_table.update_item(
        Key={'PK':f'PLAYER#{player_id}'},
        UpdateExpression="SET #clicks = if_not_exists(#clicks, :zero) + :one",
        ExpressionAttributeNames={
            '#clicks' : 'clicks'
        },
        ExpressionAttributeValues={
            ':zero' : 0,
            ':one' : 1,
            ':secret' : secret
        },
        ConditionExpression="attribute_exists(PK) AND secret = :secret",
        ReturnValues='ALL_NEW'
    ).get('Attributes')
    del player['PK']
    return player

# Update player's latency statistics using optimistic locking to prevent concurrent updates
def update_latency(player, report, secret):
    player_id = player['id']
    clicks = Decimal(report['clicks'])
    avg_latency = Decimal(report['avg_latency'])

    # Check if player already has latency stats
    has_previous_values = player.get('avg_latency') is not None
    old_avg_latency = player.get('avg_latency', Decimal(0))
    old_avg_latency_clicks = player.get('avg_latency_clicks', Decimal(0))

    # Calculate total latency from all previous clicks
    old_total_latency = old_avg_latency * old_avg_latency_clicks

    # Add new latency total to cumulative total
    new_total_latency = old_total_latency + avg_latency * clicks

    # Update total click count
    new_avg_latency_clicks = old_avg_latency_clicks + clicks
    new_avg_latency = new_total_latency / new_avg_latency_clicks

    # Prepare update values
    expression_attribute_value={
        ':new_avg_latency' : new_avg_latency,
        ':new_avg_latency_clicks' : new_avg_latency_clicks,
        ':secret' : secret
    }

    # Use different conditions for first update vs subsequent updates
    condition = (
        "attribute_exists(PK) AND secret = :secret AND #avg_latency = :old_avg_latency AND #avg_latency_clicks = :old_avg_latency_clicks"
        if has_previous_values
        else "attribute_exists(PK) AND secret = :secret AND attribute_not_exists(#avg_latency) AND attribute_not_exists(#avg_latency_clicks)"
    )
    if has_previous_values:
        expression_attribute_value[':old_avg_latency'] = old_avg_latency
        expression_attribute_value[':old_avg_latency_clicks'] = old_avg_latency_clicks

    # Perform conditional update in DynamoDB
    player = backend_table.update_item(
        Key={'PK':f'PLAYER#{player_id}'},
        UpdateExpression="SET #avg_latency = :new_avg_latency, #avg_latency_clicks = :new_avg_latency_clicks",
        ExpressionAttributeNames={
            '#avg_latency' : 'avg_latency',
            '#avg_latency_clicks' : 'avg_latency_clicks'
        },
        ExpressionAttributeValues=expression_attribute_value,
        ConditionExpression=condition,
        ReturnValues='ALL_NEW'
    ).get('Attributes')
    del player['PK']
    return player

# Handle click mutation after verifying game is in progress
def mutation_click(player_id, secret):
    game_status = get_game_status()
    if game_status is None or game_status != 'STARTED':
        raise AppSyncError('InvalidGameStatus', 'Game is not started')
    return update_click(player_id, secret)

# Handle latency report mutation after verifying game is in progress
def mutation_report_latency(player_id, report, secret):
    game_status = get_game_status()
    if game_status is None or game_status != 'STARTED':
        raise AppSyncError('InvalidGameStatus', 'Game is not started')
    player = get_player(player_id)

    return update_latency(player, report, secret)

# Process AppSync event by extracting operation type and arguments
def handle_appsync_event(event):
    args = event['arguments']
    op_type = event['info']['parentTypeName']
    op = event['info']['fieldName']

    if op_type == 'Mutation':
        if op == 'clickPython':
            return mutation_click(args['player_id'], args['secret'])
        elif op == 'reportLatencyPython':
            return mutation_report_latency(args['player_id'], args['report'], args['secret'])
        else:
            Exception('Unknown operation')

# Main Lambda handler that processes batched AppSync events and handles errors
def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    results=[]
    for appsync_event in event:
        try:
            res = handle_appsync_event(appsync_event)
            results.append({
                'data': res
            })
        except AppSyncError as ase:
            logger.exception(f"[{ase.error_type}]{ase.error_message}")
            results.append({
                'data': None,
                'error': {
                    'error_type': ase.error_type,
                    'error_message': ase.error_message
                }
            })
        except Exception as e:
            logger.exception(str(e))
            results.append({
                'data': None,
                'error': {
                    'error_message': str(e)
                }
            })
    return results
