//! HTTP server for the Battlesnake webhook API.

use axum::{
    extract::State as AxState,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use honey_badger::{api::GameState, config_for, search};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct GameMem {
    /// estimated network + engine overhead in ms
    net_ms: f64,
    /// our total processing time of the last move
    last_compute_ms: u64,
    last_seen: Instant,
}

struct AppState {
    games: Mutex<HashMap<String, GameMem>>,
    default_net_ms: f64,
    safety_ms: f64,
    max_budget_ms: f64,
    parallel: bool,
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_str(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

async fn index() -> Json<Value> {
    Json(json!({
        "apiversion": "1",
        "author": env_str("SNAKE_AUTHOR", "nickhrm"),
        "color": env_str("SNAKE_COLOR", "#E8A317"),
        "head": env_str("SNAKE_HEAD", "evil"),
        "tail": env_str("SNAKE_TAIL", "sharp"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn start(AxState(app): AxState<Arc<AppState>>, Json(gs): Json<GameState>) -> StatusCode {
    let mut g = app.games.lock().unwrap();
    // forget stale games
    g.retain(|_, m| m.last_seen.elapsed() < Duration::from_secs(1800));
    g.insert(
        gs.game.id.clone(),
        GameMem {
            net_ms: app.default_net_ms,
            last_compute_ms: 0,
            last_seen: Instant::now(),
        },
    );
    println!(
        "START game={} ruleset={} map={} {}x{} snakes={} timeout={}",
        short(&gs.game.id),
        gs.game.ruleset.name,
        gs.game.map,
        gs.board.width,
        gs.board.height,
        gs.board.snakes.len(),
        gs.game.timeout
    );
    StatusCode::OK
}

async fn end(AxState(app): AxState<Arc<AppState>>, Json(gs): Json<GameState>) -> StatusCode {
    app.games.lock().unwrap().remove(&gs.game.id);
    let alive = gs.board.snakes.iter().any(|s| s.id == gs.you.id);
    let winner = gs
        .board
        .snakes
        .first()
        .map(|s| s.name.clone())
        .unwrap_or_default();
    println!(
        "END game={} turn={} result={} winner={}",
        short(&gs.game.id),
        gs.turn,
        if alive && gs.board.snakes.len() == 1 {
            "WIN"
        } else if alive {
            "?"
        } else {
            "LOSS/DRAW"
        },
        winner
    );
    StatusCode::OK
}

fn short(id: &str) -> &str {
    &id[..id.len().min(8)]
}

async fn mv(AxState(app): AxState<Arc<AppState>>, Json(gs): Json<GameState>) -> Json<Value> {
    let t0 = Instant::now();
    let timeout = if gs.game.timeout == 0 {
        500.0
    } else {
        gs.game.timeout as f64
    };

    // update latency estimate from what the engine measured for the previous turn
    let net = {
        let mut g = app.games.lock().unwrap();
        let m = g.entry(gs.game.id.clone()).or_insert(GameMem {
            net_ms: app.default_net_ms,
            last_compute_ms: 0,
            last_seen: Instant::now(),
        });
        if let Some(lat) = gs.you.latency_ms() {
            if lat > 0 && m.last_compute_ms > 0 {
                let observed = lat as f64 - m.last_compute_ms as f64;
                let observed = observed.max(0.0);
                // fast to react to spikes, slow to relax
                m.net_ms = observed.max(0.85 * m.net_ms + 0.15 * observed);
            }
        }
        m.last_seen = Instant::now();
        m.net_ms
    };

    let budget = (timeout - net - app.safety_ms).clamp(15.0, app.max_budget_ms.min(timeout * 0.9));
    let deadline = t0 + Duration::from_millis(budget as u64);

    let parallel = app.parallel;
    let turn = gs.turn;
    let gid = gs.game.id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let st = gs.to_state();
        let cfg = config_for(&st, parallel);
        search::search(&st, &cfg, deadline)
    })
    .await;

    let elapsed = t0.elapsed().as_millis() as u64;
    if let Some(m) = app.games.lock().unwrap().get_mut(&gid) {
        m.last_compute_ms = elapsed;
    }

    match result {
        Ok(r) => {
            println!(
                "MOVE game={} turn={} move={} depth={} score={:.1} nodes={} ms={} budget={:.0} net={:.0}",
                short(&gid),
                turn,
                r.best.name(),
                r.depth,
                r.score,
                r.nodes,
                elapsed,
                budget,
                net
            );
            let shout = if search::is_win(r.score) {
                "gg"
            } else if search::is_loss(r.score) {
                "ouch"
            } else {
                ""
            };
            Json(json!({ "move": r.best.name(), "shout": shout }))
        }
        Err(e) => {
            eprintln!("search panicked: {e}");
            Json(json!({ "move": "up" }))
        }
    }
}

fn healthcheck(port: &str) -> ! {
    use std::io::{Read, Write};
    let ok = (|| -> std::io::Result<bool> {
        let mut s = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))?;
        s.set_read_timeout(Some(Duration::from_secs(2)))?;
        s.write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")?;
        let mut buf = String::new();
        s.read_to_string(&mut buf)?;
        Ok(buf.starts_with("HTTP/1.1 200") || buf.starts_with("HTTP/1.0 200"))
    })()
    .unwrap_or(false);
    std::process::exit(if ok { 0 } else { 1 })
}

#[tokio::main]
async fn main() {
    let port = env_str("PORT", "8000");
    if std::env::args().any(|a| a == "--healthcheck") {
        healthcheck(&port);
    }
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let app = Arc::new(AppState {
        games: Mutex::new(HashMap::new()),
        default_net_ms: env_f64("NET_ESTIMATE_MS", 150.0),
        safety_ms: env_f64("SAFETY_MS", 60.0),
        max_budget_ms: env_f64("MAX_BUDGET_MS", 420.0),
        parallel: std::env::var("PARALLEL")
            .map(|v| v != "0")
            .unwrap_or(cores > 1),
    });

    let router = Router::new()
        .route("/", get(index))
        .route("/start", post(start))
        .route("/move", post(mv))
        .route("/end", post(end))
        .with_state(app);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    println!(
        "honey_badger v{} listening on {addr} ({cores} cores)",
        env!("CARGO_PKG_VERSION")
    );
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            #[cfg(unix)]
            {
                let mut term =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("signal handler");
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = term.recv() => {},
                }
            }
            #[cfg(not(unix))]
            let _ = tokio::signal::ctrl_c().await;
            println!("shutting down");
        })
        .await
        .unwrap();
}
