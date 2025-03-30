import argparse
import time
import concurrent.futures
import urllib3
import json
import uuid

http = urllib3.PoolManager()

# 1 sec = 1 billion ns
ONE_SECOND=1000000000
# 1 ms = 1 million ns
ONE_MILLISECOND=1000000
def timestamp_nano():
    return time.time_ns()

def call_api(req):
    return json.loads(http.request(
        'POST',
        api_endpoint,
        headers={
            'Content-Type':'text/json',
            'x-api-key': api_key
        },
        body=bytes(
            json.dumps(
                {'query':req}
            ),
            encoding='utf8')
    ).data)
    
def register_player(player_idx):
    player_name=f"Player{player_idx}"
    player_secret=str(uuid.uuid4())
    req=f'mutation{{registerNewPlayer(name:"{player_name}",secret:"{player_secret}"){{id name team}}}}'
    player=call_api(req)['data']['registerNewPlayer']
    print(player)
    player_id=player['id']
    player_team=player['team']
    return f"{player_idx} {player_name} {player_team} {player_id} {player_secret}"

def player_line_to_player(line):
    fields=line.split(" ")
    return {
        'idx':int(fields[0]),
        'name': fields[1],
        'team': fields[2],
        'id': fields[3],
        'secret': fields[4],
    }

def get_players(requested_count, config):
    print("Verifying player registration...")
    try:
        with open(config, 'r') as f:
            lines = f.readlines()
        lines = [line.strip() for line in lines]
    except:
        lines = []

    players_in_config = len(lines)
    if players_in_config < requested_count:
        print(f"Only {players_in_config}/{requested_count} players in the config file ({config})")
        players_to_generate = requested_count - players_in_config
        print(f"Registering {players_to_generate} players...")
        threads=[]
        tasks = [executor.submit(register_player, idx) for idx in range(players_in_config + 1, requested_count + 1)]
        for task in concurrent.futures.as_completed(tasks):
            player_line = task.result()
            lines.append(player_line)
        with open(config, 'w') as f:
            f.write("\n".join(lines))
    return [
        player_line_to_player(line)
        for line in sorted(lines[:requested_count])
    ]

def click_mutation_for_team(team):
    if team == 'RUST':
        return 'clickRust'
    elif team == 'PYTHON':
        return 'clickPython'
    elif team == 'JS':
        return 'clickJs'
    elif team == 'VTL':
        return 'clickVtl'
    raise(Exception(f"Team unknown: {team}"))
def report_mutation_for_team(team):
    if team == 'RUST':
        return 'reportLatencyRust'
    elif team == 'PYTHON':
        return 'reportLatencyPython'
    elif team == 'JS':
        return 'reportLatencyJs'
    elif team == 'VTL':
        return 'reportLatencyVtl'
    raise(Exception(f"Team unknown: {team}"))

def metered_click(req, reports):
    start=timestamp_nano()
    call_api(req)
    end=timestamp_nano()
    print(f"Click took {(end-start)/ONE_MILLISECOND}ms")
    reports.append((end-start)/ONE_MILLISECOND)
def report_latency(req):
    if call_api(req).get('errors') is not None:
        print("Error reporting. Is the game started??")

def player_play(player, click_freq, duration):
    player_id=player['id']
    player_secret=player['secret']
    player_name=player['name']
    print(f"Starting player {player_name}({player_id})")

    click_mutation=click_mutation_for_team(player['team'])
    report_mutation=report_mutation_for_team(player['team'])
    click_req=f'mutation{{{click_mutation}(player_id:"{player_id}",secret:"{player_secret}"){{id name team clicks avg_latency avg_latency_clicks}}}}'

    stop_player_at=timestamp_nano() + duration * ONE_SECOND
    # next_report in 1 second
    next_report=timestamp_nano() + ONE_SECOND
    reports=[]
    while True:
        will_stop=timestamp_nano() > stop_player_at
        next_click_ts=timestamp_nano() + ONE_SECOND/click_freq

        if not will_stop:
            executor.submit(metered_click, click_req, reports)

        if timestamp_nano() > next_report or will_stop:
            # REPORT
            to_report=[]
            while len(reports) > 0:
                to_report.append(reports.pop())
            clicks=len(to_report)
            if clicks > 0:
                avg=sum(to_report)/clicks
                report=f"{{clicks:{clicks},avg_latency:{avg}}}"
                print(f"{player_name}({player_id}): {report}")
                report_req=f'mutation{{{report_mutation}(player_id:"{player_id}",report:{report},secret:"{player_secret}"){{id name team clicks avg_latency avg_latency_clicks}}}}'
                executor.submit(report_latency, report_req)
            # next_report in 1 second
            next_report=timestamp_nano() + ONE_SECOND
        if will_stop:
            break
        
        now=timestamp_nano()
        if now < next_click_ts:
            time.sleep((next_click_ts-now)/ONE_SECOND)
    
    print(f"Stopping player {player_name}({player_id})")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog='simulate_players',
        description='Simulate players playing the Benchmark Game. By default, simulate 100 players that click 7 times/second. The script will onboard each player and store the infos in a text file for further usage.',
    )

    parser.add_argument('--api-endpoint', required=True)
    parser.add_argument('--api-key', required=True)
    parser.add_argument('-p', '--players', default=100, type=int)
    parser.add_argument('-f', '--frequency', default=7, type=int)
    parser.add_argument('-d', '--duration', default=20, type=int)
    parser.add_argument('-c', '--config', default="./simulate_players.config.txt")
    parser.add_argument('--register-only', action='store_true')
    args = parser.parse_args()

    global api_endpoint
    api_endpoint = args.api_endpoint
    global api_key
    api_key = args.api_key

    global executor
    executor = concurrent.futures.ThreadPoolExecutor(max_workers=args.players*10)

    players = get_players(args.players, args.config)

    if args.register_only:
        exit(0)
    tasks = [executor.submit(player_play, player, args.frequency, args.duration) for player in players]
    
    for task in concurrent.futures.as_completed(tasks):
        pass
    
    executor.shutdown()
