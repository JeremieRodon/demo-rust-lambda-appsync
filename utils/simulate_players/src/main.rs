use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{
    error::Error,
    sync::Arc,
    time::{Duration, Instant},
};

fn e_to_s(e: impl Error) -> String {
    log::error!("{e}");
    e.to_string()
}

#[derive(Debug, Clone)]
struct ApiCaller {
    client: Client,
    url: Arc<str>,
    key: Arc<str>,
}
impl ApiCaller {
    fn new(url: String, key: String) -> Self {
        ApiCaller {
            client: reqwest::Client::new(),
            url: Arc::from(url),
            key: Arc::from(key),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct PlayerConfig {
    idx: usize,
    name: String,
    team: String,
    id: String,
    secret: String,
}

impl From<&str> for PlayerConfig {
    fn from(value: &str) -> Self {
        let mut splited = value.split(" ");
        PlayerConfig {
            idx: splited.next().unwrap().parse().unwrap(),
            name: splited.next().unwrap().to_owned(),
            team: splited.next().unwrap().to_owned(),
            id: splited.next().unwrap().to_owned(),
            secret: splited.next().unwrap().to_owned(),
        }
    }
}
impl core::fmt::Display for PlayerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.idx, self.name, self.team, self.id, self.secret
        )
    }
}

async fn call_api(api: ApiCaller, api_req: &str) -> Result<serde_json::Value, String> {
    let body = json!({
        "query": api_req
    });
    let req = api
        .client
        .post(api.url.as_ref())
        .header("Content-Type", "text/json")
        .header("x-api-key", api.key.as_ref())
        .body(serde_json::to_string(&body).unwrap());
    let resp = req.send().await.map_err(e_to_s)?;
    let body_bytes = resp.bytes().await.map_err(e_to_s)?.into();
    let body_str = String::from_utf8(body_bytes).map_err(e_to_s)?;
    let mut resp_value: serde_json::Value = serde_json::from_str(&body_str).map_err(e_to_s)?;
    if let Some(errors) = resp_value.get("errors") {
        log::error!("{}", serde_json::to_string_pretty(errors).unwrap());
        Ok(resp_value)
    } else {
        Ok(resp_value.get_mut("data").unwrap().take())
    }
}

async fn register_player(api: ApiCaller, idx: usize) -> Result<PlayerConfig, String> {
    let player_name = format!("Player{idx}");
    let player_secret = uuid::Uuid::new_v4().to_string();
    let req = format!(
        "mutation{{registerNewPlayer(name:\"{player_name}\",secret:\"{player_secret}\"){{id name team}}}}"
    );
    let new_player = call_api(api.clone(), &req)
        .await?
        .get_mut("registerNewPlayer")
        .unwrap()
        .take();

    log::debug!("{new_player}");

    Ok(PlayerConfig {
        idx,
        name: player_name,
        team: new_player.get("team").unwrap().as_str().unwrap().to_owned(),
        id: new_player.get("id").unwrap().as_str().unwrap().to_owned(),
        secret: player_secret,
    })
}

async fn get_players(
    api: ApiCaller,
    requested_count: usize,
    config_file: String,
) -> Result<Vec<PlayerConfig>, String> {
    log::info!("Verifying player registration...");
    let mut players = std::fs::read_to_string(&config_file)
        .map_err(e_to_s)
        .unwrap_or_default()
        .split_terminator("\n")
        .inspect(|&s| log::debug!("{s}"))
        .map(PlayerConfig::from)
        .collect::<Vec<_>>();

    if players.len() < requested_count {
        log::info!(
            "Only {}/{requested_count} players in the config file ({config_file})",
            players.len()
        );
        let players_to_generate = requested_count - players.len();
        log::info!("Registering {players_to_generate} players...");
        let tasks = ((players.len() + 1)..=(requested_count))
            .map(|idx| tokio::spawn(register_player(api.clone(), idx)))
            .collect::<Vec<_>>();
        for t in tasks {
            let player = t.await.unwrap()?;
            players.push(player);
        }
        players.sort_by_key(|p| p.idx);
        let content = players.iter().map(|p| format!("{p}\n")).collect::<String>();
        std::fs::write(&config_file, content).map_err(e_to_s)?;
    }

    Ok(players)
}

fn click_mutation_for_team(team: &str) -> &'static str {
    match team {
        "RUST" => "clickRust",
        "PYTHON" => "clickPython",
        "JS" => "clickJs",
        "VTL" => "clickVtl",
        _ => panic!("Unknown team: {team}"),
    }
}
fn report_mutation_for_team(team: &str) -> &'static str {
    match team {
        "RUST" => "reportLatencyRust",
        "PYTHON" => "reportLatencyPython",
        "JS" => "reportLatencyJs",
        "VTL" => "reportLatencyVtl",
        _ => panic!("Unknown team: {team}"),
    }
}

async fn metered_click(
    api: ApiCaller,
    req: Arc<str>,
    report_pipe: tokio::sync::mpsc::UnboundedSender<u128>,
) -> Result<(), String> {
    let start = Instant::now();
    call_api(api, &req).await?;
    let end = Instant::now();
    let duration_milli = (end - start).as_millis();
    log::debug!("Call took {duration_milli}ms");
    report_pipe.send(duration_milli).map_err(e_to_s)?;
    Ok(())
}

async fn report_latency(api: ApiCaller, req: &str) -> Result<(), String> {
    let res = call_api(api, req).await?;
    if res.get("errors").is_some() {
        log::error!("Error reporting. Is the game started??");
    }
    Ok(())
}

async fn latency_reporter(
    api: ApiCaller,
    player: PlayerConfig,
    mut report_pipe: tokio::sync::mpsc::UnboundedReceiver<u128>,
) -> Result<(), String> {
    let PlayerConfig {
        idx: _,
        name,
        team,
        id,
        secret,
    } = player;
    log::info!("Starting latency reporter for {name}({id})");
    let report_mutation = report_mutation_for_team(&team);
    let mut latencies: Vec<u128> = Vec::new();
    // next_report in 1 second
    let mut next_report = Instant::now() + Duration::from_secs(1);

    while let Some(latency) = report_pipe.recv().await {
        let now = Instant::now();
        latencies.push(latency);

        if now > next_report {
            let clicks = latencies.len();
            if clicks > 0 {
                let avg = latencies.iter().sum::<u128>() as f64 / clicks as f64;
                let click_req = format!(
                    "mutation{{{report_mutation}(player_id:\"{id}\",report:{{clicks:{clicks},avg_latency:{avg}}},secret:\"{secret}\"){{id name team clicks avg_latency avg_latency_clicks}}}}"
                );
                report_latency(api.clone(), &click_req).await?;
            }
            next_report = now + Duration::from_secs(1);
        }
    }
    log::info!("Stopping latency reporter for {name}({id})");
    Ok(())
}

async fn player_play(
    api: ApiCaller,
    player: PlayerConfig,
    click_freq: u64,
    duration: u64,
) -> Result<(), String> {
    let PlayerConfig {
        idx: _,
        name,
        team,
        id,
        secret,
    } = player.clone();
    log::info!("Starting {name}({id})");

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(latency_reporter(api.clone(), player, receiver));

    let click_mutation = click_mutation_for_team(&team);
    let click_req: Arc<str> = Arc::from(format!(
        "mutation{{{click_mutation}(player_id:\"{id}\",secret:\"{secret}\"){{id name team clicks avg_latency avg_latency_clicks}}}}"
    ));

    let stop_player_at = Instant::now() + Duration::from_secs(duration);

    loop {
        let now = Instant::now();
        let next_click =
            now + Duration::from_nanos(Duration::from_secs(1).as_nanos() as u64 / click_freq);
        if now < stop_player_at {
            tokio::spawn(metered_click(
                api.clone(),
                click_req.clone(),
                sender.clone(),
            ));
        } else {
            break;
        }
        tokio::time::sleep_until(next_click.into()).await;
    }

    log::info!("Stopping {name}({id})");
    Ok(())
}

#[derive(Clone, Debug, Parser)]
#[command(author= option_env ! ("CARGO_PKG_AUTHORS").unwrap_or(""), version = option_env ! ("CARGO_PKG_VERSION").unwrap_or("unknown"), about, long_about = None)]
pub struct CliParser {
    #[arg(long)]
    pub api_endpoint: String,
    #[arg(long)]
    pub api_key: String,
    #[arg(short, long, default_value_t = 100)]
    pub players: usize,
    #[arg(short, long, default_value_t = 7)]
    pub frequency: u64,
    #[arg(short, long, default_value_t = 20)]
    pub duration: u64,
    #[arg(short, long, default_value = "./simulate_players.config.txt")]
    pub config: String,
    #[arg(long, default_value_t = false)]
    pub register_only: bool,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug,tracing::span=warn"),
    )
    .format_timestamp_micros()
    .init();

    let cli_parser = CliParser::parse();
    log::debug!("Processing {:?}", cli_parser);
    let api = ApiCaller::new(cli_parser.api_endpoint, cli_parser.api_key);

    let Ok(players) = get_players(api.clone(), cli_parser.players, cli_parser.config).await else {
        return ();
    };
    if cli_parser.register_only {
        return ();
    }

    let player_tasks = players
        .into_iter()
        .map(|p| {
            tokio::spawn(player_play(
                api.clone(),
                p,
                cli_parser.frequency,
                cli_parser.duration,
            ))
        })
        .collect::<Vec<_>>();
    for player_task in player_tasks {
        match player_task.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                log::error!("{e}")
            }
        }
    }
}
