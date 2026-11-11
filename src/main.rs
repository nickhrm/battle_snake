//! HTTP server for the Battlesnake webhook API.

use axum::{
    extract::{Path, State as AxState},
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
    /// answer this much sooner while a timeout would be costly (the engine repeats our last move)
    early_ms: f64,
    /// think this much longer while the timeout move is our choice anyway
    late_ms: f64,
    parallel: bool,
    /// experiment variants by name, served under /v/<name>/ (see `honey_badger::apply_variant`)
    variants: HashMap<String, Value>,
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

async fn index_variant(AxState(app): AxState<Arc<AppState>>, Path(name): Path<String>) -> Result<Json<Value>, StatusCode> {
    let v = app.variants.get(&name).ok_or(StatusCode::NOT_FOUND)?;
    let Json(mut info) = index().await;
    for key in ["color", "head", "tail"] {
        if let Some(x) = v.get(key) {
            info[key] = x.clone();
        }
    }
    Ok(Json(info))
}

async fn start_variant(app: AxState<Arc<AppState>>, Path(_): Path<String>, gs: Json<GameState>) -> StatusCode {
    start(app, gs).await
}

async fn end_variant(app: AxState<Arc<AppState>>, Path(_): Path<String>, gs: Json<GameState>) -> StatusCode {
    end(app, gs).await
}

async fn mv_variant(AxState(app): AxState<Arc<AppState>>, Path(name): Path<String>, Json(gs): Json<GameState>) -> Result<Json<Value>, StatusCode> {
    let v = app.variants.get(&name).cloned().ok_or(StatusCode::NOT_FOUND)?;
    Ok(play(app, gs, Some((name, v))).await)
}

async fn mv(AxState(app): AxState<Arc<AppState>>, Json(gs): Json<GameState>) -> Json<Value> {
    play(app, gs, None).await
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
        mem_key(&gs),
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
    app.games.lock().unwrap().remove(&mem_key(&gs));
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

/// Two of our snakes (production and a variant) can meet in one game: keep their latency
/// estimates apart.
fn mem_key(gs: &GameState) -> String {
    format!("{}:{}", gs.game.id, gs.you.id)
}

fn short(id: &str) -> &str {
    &id[..id.len().min(8)]
}

async fn play(app: Arc<AppState>, gs: GameState, variant: Option<(String, Value)>) -> Json<Value> {
    let t0 = Instant::now();
    let timeout = if gs.game.timeout == 0 {
        500.0
    } else {
        gs.game.timeout as f64
    };

    // update latency estimate from what the engine measured for the previous turn
    let net = {
        let mut g = app.games.lock().unwrap();
        let m = g.entry(mem_key(&gs)).or_insert(GameMem {
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

    // room for the extension: stay 40 ms below the timeout after the network overhead
    let late = app.late_ms.min(timeout - net - 40.0 - budget).max(0.0);
    let early = app.early_ms.min(budget * 0.5).max(0.0);
    let parallel = app.parallel;
    let turn = gs.turn;
    let tag = variant.as_ref().map(|(n, _)| format!(" v={n}")).unwrap_or_default();
    let variant_cfg = variant;
    let gid = gs.game.id.clone();
    let key = mem_key(&gs);
    let result = tokio::task::spawn_blocking(move || {
        let st = gs.to_state();
        let mut cfg = config_for(&st, parallel);
        if let Some((_, v)) = &variant_cfg {
            honey_badger::apply_variant(&mut cfg, &st, v);
        }
        cfg.fallback = Some(gs.you.last_move());
        cfg.early = Duration::from_millis(early as u64);
        cfg.late = Duration::from_millis(late as u64);
        search::search(&st, &cfg, deadline)
    })
    .await;

    let elapsed = t0.elapsed().as_millis() as u64;
    if let Some(m) = app.games.lock().unwrap().get_mut(&key) {
        m.last_compute_ms = elapsed;
    }

    match result {
        Ok(r) => {
            println!(
                "MOVE game={} turn={} move={} depth={} score={:.1} nodes={} ms={} budget={:.0} net={:.0}{tag}",
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
        max_budget_ms: env_f64("MAX_BUDGET_MS", 360.0),
        early_ms: env_f64("EARLY_MS", 100.0),
        late_ms: env_f64("LATE_MS", 60.0),
        parallel: std::env::var("PARALLEL")
            .map(|v| v != "0")
            .unwrap_or(cores > 1),
        variants: std::env::var("SNAKE_VARIANTS")
            .ok()
            .map(|js| serde_json::from_str(&js).expect("SNAKE_VARIANTS must be a JSON object of variants"))
            .unwrap_or_default(),
    });
    for name in app.variants.keys() {
        println!("variant /v/{name}/");
    }

    let router = Router::new()
        .route("/", get(index))
        .route("/start", post(start))
        .route("/move", post(mv))
        .route("/end", post(end))
        .route("/v/:name", get(index_variant))
        .route("/v/:name/", get(index_variant))
        .route("/v/:name/start", post(start_variant))
        .route("/v/:name/move", post(mv_variant))
        .route("/v/:name/end", post(end_variant))
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
