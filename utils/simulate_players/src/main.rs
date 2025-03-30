//! This binary simulates multiple players participating in a GraphQL API benchmark game.
//!
//! The game mechanics are simple - players click a button as fast as they can and the API
//! measures response times. Each player is randomly assigned to one of four teams, where
//! each team uses a different backend implementation (Rust, Python, JavaScript, or VTL).
//!
//! # Usage
//!
//! First register the simulated players (required once):
//! ```bash
//! ./simulate_players \
//!     --api-endpoint "https://xxxxx.appsync-api.region.amazonaws.com/graphql" \
//!     --api-key "da2-xxxxxxxxxxxxxxxxxxxx" \
//!     --players 10 \
//!     --register-only
//! ```
//!
//! Then run the simulation to generate load (can be run multiple times):
//! ```bash
//! ./simulate_players \
//!     --api-endpoint "https://xxxxx.appsync-api.region.amazonaws.com/graphql" \
//!     --api-key "da2-xxxxxxxxxxxxxxxxxxxx" \
//!     --players 10 \        # Number of concurrent players to simulate
//!     --frequency 7 \       # How many clicks per second per player
//!     --duration 20         # How long to run the simulation in seconds
//! ```
//!
//! The simulation will:
//! 1. Load previously registered players from the config file (or register new ones if needed)
//! 2. Start a task for each simulated player that will:
//!    - Click at the specified frequency by calling the appropriate GraphQL mutation
//!    - Measure response time for each click
//!    - Report average latency statistics every second
//! 3. Run until the specified duration has elapsed
//!
//! # Notes
//!
//! - Players are persisted in `simulate_players.config.txt` (override with --config)
//! - Each player has a unique ID and secret key used to authenticate their actions
//! - The simulation tries to maintain consistent click timing to match the requested frequency
//! - Failed API calls and GraphQL errors are logged but don't stop the simulation
//! - If you need to "reset" the simulation after removing all players, just delete the `simulate_players.config.txt` file

/// Imports required for this binary which simulates multiple players in the AppSync GraphQL Benchmark Game
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{
    error::Error,
    sync::Arc,
    time::{Duration, Instant},
};

/// Helper function to convert any Error into a String, while also logging it
///
/// Returns:
///   The error message as a String
fn e_to_s(e: impl Error) -> String {
    log::error!("{e}");
    log::error!("{e:?}");
    e.to_string()
}

/// Struct holding everything needed to make API calls to the GraphQL endpoint
#[derive(Debug, Clone)]
struct ApiCaller {
    /// The HTTP client used to make the requests
    client: Client,
    /// The URL of the GraphQL API endpoint
    url: Arc<str>,
    /// The API key used for authentication
    key: Arc<str>,
}
impl ApiCaller {
    /// Creates a new ApiCaller instance for the given API endpoint
    ///
    /// Returns:
    ///   A new ApiCaller ready to make requests
    fn new(url: String, key: String) -> Self {
        ApiCaller {
            client: reqwest::Client::new(),
            // Use Arc<str> to efficiently share these strings between async tasks
            url: Arc::from(url),
            key: Arc::from(key),
        }
    }
}

/// Configuration for a single simulated player
#[derive(Debug, Clone, Deserialize)]
struct PlayerConfig {
    /// Sequential index of the player
    idx: usize,
    /// Player's display name
    name: String,
    /// The team assigned to this player (RUST, PYTHON, JS or VTL)
    team: String,
    /// Unique identifier for this player
    id: String,
    /// Secret key used to authenticate player actions
    secret: String,
}

impl From<&str> for PlayerConfig {
    /// Creates a PlayerConfig from a space-separated string containing all fields
    ///
    /// Format: "{idx} {name} {team} {id} {secret}"
    ///
    /// # Panics
    ///
    /// Panics if the string does not contain exactly 5 space-separated values
    /// or if the first value cannot be parsed as usize
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
    /// Formats the PlayerConfig back into the space-separated string format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.idx, self.name, self.team, self.id, self.secret
        )
    }
}

/// Makes a GraphQL API call and returns the response data
///
/// Returns:
///   The "data" field of the GraphQL response on success, or the full response if there were errors
///
/// # Notes
///
/// - Logs any GraphQL errors that occur
/// - Handles converting between JSON and the wire format
async fn call_api(api: ApiCaller, api_req: &str) -> Result<serde_json::Value, String> {
    // Create the GraphQL request body
    let body = json!({
        "query": api_req
    });
    // Build and send the HTTP request
    let req = api
        .client
        .post(api.url.as_ref())
        .header("Content-Type", "text/json")
        .header("x-api-key", api.key.as_ref())
        .body(serde_json::to_string(&body).unwrap());
    let resp = req.send().await.map_err(e_to_s)?;
    // Read and parse the response
    let body_bytes = resp.bytes().await.map_err(e_to_s)?.into();
    let body_str = String::from_utf8(body_bytes).map_err(e_to_s)?;
    let mut resp_value: serde_json::Value = serde_json::from_str(&body_str).map_err(e_to_s)?;
    // Handle GraphQL errors by returning the full response
    if let Some(errors) = resp_value.get("errors") {
        log::error!("{}", serde_json::to_string_pretty(errors).unwrap());
        Ok(resp_value)
    } else {
        Ok(resp_value.get_mut("data").unwrap().take())
    }
}

/// Registers a new player with the API and returns their configuration
///
/// Returns:
///   The PlayerConfig for the newly registered player
///
/// # Notes
///
/// The player will be randomly assigned to one of the teams (RUST, PYTHON, JS, VTL)
async fn register_player(api: ApiCaller, idx: usize) -> Result<PlayerConfig, String> {
    let player_name = format!("Player{idx}");
    let player_secret = uuid::Uuid::new_v4().to_string();
    // Send the registerNewPlayer mutation
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

/// Gets or creates the requested number of players
///
/// Returns:
///   A vector containing the PlayerConfig for each player
///
/// # Notes
///
/// - Reads existing players from the config file
/// - Registers new players if needed to reach the requested count
/// - Saves the updated player list back to the config file
async fn get_players(
    api: ApiCaller,
    requested_count: usize,
    config_file: String,
) -> Result<Vec<PlayerConfig>, String> {
    log::info!("Verifying player registration...");
    // Try to read existing players from the config file
    let mut players = std::fs::read_to_string(&config_file)
        .map_err(e_to_s)
        .unwrap_or_default()
        .split_terminator("\n")
        .take(requested_count)
        .inspect(|&s| log::debug!("{s}"))
        .map(PlayerConfig::from)
        .collect::<Vec<_>>();

    // Register additional players if needed
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
        // Sort by index and save to config file
        players.sort_by_key(|p| p.idx);
        let content = players.iter().map(|p| format!("{p}\n")).collect::<String>();
        std::fs::write(&config_file, content).map_err(e_to_s)?;
    }

    Ok(players)
}

/// Returns the appropriate click mutation name for a given team
///
/// # Panics
///
/// Panics if the team name is not one of: RUST, PYTHON, JS, VTL
fn click_mutation_for_team(team: &str) -> &'static str {
    match team {
        "RUST" => "clickRust",
        "PYTHON" => "clickPython",
        "JS" => "clickJs",
        "VTL" => "clickVtl",
        _ => panic!("Unknown team: {team}"),
    }
}

/// Returns the appropriate latency report mutation name for a given team
///
/// # Panics
///
/// Panics if the team name is not one of: RUST, PYTHON, JS, VTL
fn report_mutation_for_team(team: &str) -> &'static str {
    match team {
        "RUST" => "reportLatencyRust",
        "PYTHON" => "reportLatencyPython",
        "JS" => "reportLatencyJs",
        "VTL" => "reportLatencyVtl",
        _ => panic!("Unknown team: {team}"),
    }
}

/// Makes a click API call and measures its latency
///
/// Returns:
///   Nothing on success, error message on failure
///
/// # Notes
///
/// The measured latency is sent through the report_pipe channel
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

/// Reports accumulated latency statistics to the API
///
/// Returns:
///   Nothing on success, error message on failure
async fn report(
    api: ApiCaller,
    report_mutation: &str,
    id: &str,
    secret: &str,
    latencies: &mut Vec<u128>,
) -> Result<(), String> {
    let clicks = latencies.len();
    if clicks > 0 {
        // Calculate average latency
        let avg = latencies.iter().sum::<u128>() as f64 / clicks as f64;
        // Send the report mutation
        let click_req = format!(
            "mutation{{{report_mutation}(player_id:\"{id}\",report:{{clicks:{clicks},avg_latency:{avg}}},secret:\"{secret}\"){{id name team clicks avg_latency avg_latency_clicks}}}}"
        );
        if call_api(api.clone(), &click_req)
            .await?
            .get("errors")
            .is_some()
        {
            log::error!("Error reporting. Is the game started??");
        }
    }
    // Clear accumulated latencies
    latencies.clear();
    Ok(())
}

/// Task that receives latency measurements and periodically reports them
///
/// Returns:
///   Nothing on success, error message on failure
///
/// # Notes
///
/// Reports are sent every second, plus one final report when the channel closes
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
    // Schedule next report in 1 second
    let mut next_report = Instant::now() + Duration::from_secs(1);

    // Process latencies until the channel closes
    while let Some(latency) = report_pipe.recv().await {
        let now = Instant::now();
        latencies.push(latency);

        // Send report if it's time
        if now > next_report {
            report(api.clone(), report_mutation, &id, &secret, &mut latencies).await?;
            next_report = now + Duration::from_secs(1);
        }
    }
    // Send final report
    report(api, report_mutation, &id, &secret, &mut latencies).await?;
    log::info!("Stopped latency reporter for {name}({id})");
    Ok(())
}

/// Simulates a single player clicking at a specified frequency for a given duration
///
/// Returns:
///   Nothing on success, error message on failure
///
/// # Notes
///
/// - Spawns a latency reporter task to handle statistics
/// - Each click is made in its own async task
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

    // Setup channel for latency reporting
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let latency_reporter = tokio::spawn(latency_reporter(api.clone(), player, receiver));

    // Prepare click mutation
    let click_mutation = click_mutation_for_team(&team);
    let click_req: Arc<str> = Arc::from(format!(
        "mutation{{{click_mutation}(player_id:\"{id}\",secret:\"{secret}\"){{id name team clicks avg_latency avg_latency_clicks}}}}"
    ));

    // Calculate when to stop clicking
    let stop_player_at = Instant::now() + Duration::from_secs(duration);

    // Click loop
    loop {
        let now = Instant::now();
        // Calculate when to make next click based on frequency
        let next_click =
            now + Duration::from_nanos(Duration::from_secs(1).as_nanos() as u64 / click_freq);
        if now < stop_player_at {
            // Spawn click task
            tokio::spawn(metered_click(
                api.clone(),
                click_req.clone(),
                sender.clone(),
            ));
        } else {
            break;
        }
        // Wait until it's time for next click
        tokio::time::sleep_until(next_click.into()).await;
    }

    // Close channel and wait for final report
    drop(sender);
    latency_reporter.await.unwrap()?;

    log::info!("Stopped {name}({id})");
    Ok(())
}

/// Command line arguments parser
#[derive(Clone, Debug, Parser)]
#[command(author= option_env ! ("CARGO_PKG_AUTHORS").unwrap_or(""), version = option_env ! ("CARGO_PKG_VERSION").unwrap_or("unknown"), about, long_about = None)]
pub struct CliParser {
    /// URL of the GraphQL API endpoint
    #[arg(long)]
    pub api_endpoint: String,
    /// API key for authentication
    #[arg(long)]
    pub api_key: String,
    /// Number of players to simulate (default: 100)
    #[arg(short, long, default_value_t = 100)]
    pub players: usize,
    /// Clicks per second for each player (default: 7)
    #[arg(short, long, default_value_t = 7)]
    pub frequency: u64,
    /// How long to run the simulation in seconds (default: 20)
    #[arg(short, long, default_value_t = 20)]
    pub duration: u64,
    /// Path to the config file storing player information
    #[arg(short, long, default_value = "./simulate_players.config.txt")]
    pub config: String,
    /// Only register players, don't start simulation
    #[arg(long, default_value_t = false)]
    pub register_only: bool,
}

/// Main entry point - parses args and runs the simulation
#[tokio::main]
async fn main() {
    // Setup logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,tracing::span=warn"),
    )
    .format_timestamp_micros()
    .init();

    // Parse command line arguments
    let cli_parser = CliParser::parse();
    log::debug!("Processing {:?}", cli_parser);
    let api = ApiCaller::new(cli_parser.api_endpoint, cli_parser.api_key);

    // Get or create required players
    let Ok(players) = get_players(api.clone(), cli_parser.players, cli_parser.config).await else {
        return;
    };
    if cli_parser.register_only {
        return;
    }

    // Start all player tasks
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
    // Wait for all players to finish
    for player_task in player_tasks {
        match player_task.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                log::error!("{e}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_config_from_str() {
        let config_str = "1 TestPlayer RUST player123 secret456";
        let config = PlayerConfig::from(config_str);
        assert_eq!(config.idx, 1);
        assert_eq!(config.name, "TestPlayer");
        assert_eq!(config.team, "RUST");
        assert_eq!(config.id, "player123");
        assert_eq!(config.secret, "secret456");
    }

    #[test]
    fn test_player_config_display() {
        let config = PlayerConfig {
            idx: 1,
            name: "TestPlayer".to_string(),
            team: "RUST".to_string(),
            id: "player123".to_string(),
            secret: "secret456".to_string(),
        };
        assert_eq!(config.to_string(), "1 TestPlayer RUST player123 secret456");
    }

    #[test]
    fn test_click_mutation_for_team() {
        assert_eq!(click_mutation_for_team("RUST"), "clickRust");
        assert_eq!(click_mutation_for_team("PYTHON"), "clickPython");
        assert_eq!(click_mutation_for_team("JS"), "clickJs");
        assert_eq!(click_mutation_for_team("VTL"), "clickVtl");
    }

    #[test]
    #[should_panic(expected = "Unknown team: INVALID")]
    fn test_click_mutation_for_invalid_team() {
        click_mutation_for_team("INVALID");
    }

    #[test]
    fn test_report_mutation_for_team() {
        assert_eq!(report_mutation_for_team("RUST"), "reportLatencyRust");
        assert_eq!(report_mutation_for_team("PYTHON"), "reportLatencyPython");
        assert_eq!(report_mutation_for_team("JS"), "reportLatencyJs");
        assert_eq!(report_mutation_for_team("VTL"), "reportLatencyVtl");
    }

    #[test]
    #[should_panic(expected = "Unknown team: INVALID")]
    fn test_report_mutation_for_invalid_team() {
        report_mutation_for_team("INVALID");
    }
}
