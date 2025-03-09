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
    return backend_table.update_item(
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

def update_latency():
    pass

def mutation_click(player_id):
    game_status = get_game_status()
    if game_status is None or game_status != 'STARTED':
        raise AppSyncError('InvalidGameStatus', 'Game is not started')
    return update_click(player_id)

def mutation_report_latency(player_id, report):
    clicks = report['clicks']
    avg_latency = report['avg_latency']
    
    game_status = get_game_status()
    if game_status is None or game_status != 'STARTED':
        raise AppSyncError('InvalidGameStatus', 'Game is not started')
    player = get_player(player_id)

    pass

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
