import json
import os

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


class AppSyncError(Exception):
    def __init__(self, error_type, error_message):
        self.error_type = error_type
        self.error_message = error_message

def get_game_status():
    return (backend_table.get_item(Key={'PK':'GAME_STATUS'})
            .get('Item', {})
            .get('game_status'))

def get_player(player_id):
    player = (backend_table
            .get_item(Key={'PK':f'PLAYER#{player_id}'})
            .get('Item'))
    if player is None:
        return None
    del player['PK']
    return player

def update_click(player_id):
    player = backend_table.update_item(
        Key={'PK':f'PLAYER#{player_id}'},
        UpdateExpression="SET #clicks = if_not_exists(#clicks, :zero) + :one",
        ExpressionAttributeNames={
            '#clicks' : 'clicks'
        },
        ExpressionAttributeValues={
            ':zero' : 0,
            ':one' : 1
        },
        ConditionExpression="attribute_exists(PK)",
        ReturnValues='ALL_NEW'
    ).get('Attributes')
    del player['PK']
    return player

def update_latency(player, report):
    player_id = player.id
    clicks = report['clicks']
    avg_latency = report['avg_latency']
    
    has_previous_values = player.get('avg_latency') is not None
    old_avg_latency = player.get('avg_latency', 0.0)
    old_avg_latency_clicks = player.get('avg_latency_clicks', 0)
    
    old_total_latency = old_avg_latency * old_avg_latency_clicks

    new_total_latency = old_total_latency + avg_latency * clicks
    
    new_avg_latency_clicks = old_avg_latency_clicks + clicks
    new_avg_latency = new_total_latency / new_avg_latency_clicks

    expression_attribute_value={
        ':new_avg_latency' : new_avg_latency,
        ':new_avg_latency_clicks' : new_avg_latency_clicks
    }
    condition = (
        "attribute_exists(PK) AND #avg_latency = :old_avg_latency AND #avg_latency_clicks = :old_avg_latency_clicks"
        if has_previous_values
        else "attribute_exists(PK) AND attribute_not_exists(#avg_latency) AND attribute_not_exists(#avg_latency_clicks)"
    )
    if has_previous_values:
        expression_attribute_value[':old_avg_latency'] = old_avg_latency
        expression_attribute_value[':old_avg_latency_clicks'] = old_avg_latency_clicks

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

def mutation_click(player_id):
    game_status = get_game_status()
    if game_status is None or game_status != 'STARTED':
        raise AppSyncError('InvalidGameStatus', 'Game is not started')
    return update_click(player_id)

def mutation_report_latency(player_id, report):
    game_status = get_game_status()
    if game_status is None or game_status != 'STARTED':
        raise AppSyncError('InvalidGameStatus', 'Game is not started')
    player = get_player(player_id)

    return update_latency(player, report)

def handle_appsync_event(event):
    args = event['arguments']
    op_type = event['info']['parentTypeName']
    op = event['info']['fieldName']

    if op_type == 'Mutation':
        if op == 'clickPython':
            return mutation_click(args['player_id'])
        elif op == 'reportLatencyPython':
            return mutation_report_latency(args['player_id'], args['report'])
        else:
            Exception('Unknown operation')

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
