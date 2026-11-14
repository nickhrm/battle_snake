#![allow(clippy::needless_range_loop)]
pub mod api;
pub mod eval;
pub mod game;
pub mod replay;
pub mod search;

use eval::Weights;
use game::State;
use search::SearchConfig;

/// Weights for the current game type. Can be overridden with the
/// `SNAKE_WEIGHTS` environment variable (JSON, partial is fine).
pub fn weights_for(st: &State) -> Weights {
    if let Ok(js) = std::env::var("SNAKE_WEIGHTS") {
        if let Ok(w) = serde_json::from_str::<Weights>(&js) {
            return w;
        }
    }
    profile(st)
}

pub fn profile(st: &State) -> Weights {
    let w = Weights::default();
    let opps = st.snakes.len().saturating_sub(1);
    if st.rules.constrictor {
        return Weights {
            length: 0.0,
            longest_bonus: 0.0,
            food_dist: 0.0,
            eat_early: 0.0,
            // the game is decided by territory alone within about 35 turns
            opp_area: 1.0,
            opp_area_sum: 0.3,
            parity: 0.0,
            ..w
        };
    }
    if st.rules.royale {
        // the board shrinks and hazard turns cost 15 health, so a snake that waits starves
        return Weights {
            edge: 3.0,
            trap: 12.0,
            hunger_health: 90,
            health: 0.3,
            ..w
        };
    }
    if opps <= 1 {
        // duel: the value of length fades with the turn number (half-life 150 turns, at least
        // 30 %). A long body late in the game costs room, and the duels we lost late were
        // mostly lost with a big length lead.
        //
        // Territory does not count while the snakes are short (total length 8) and counts
        // fully from total length 24. Against the top duel snakes we left food on the edge
        // that was closer to us, because the way there costs territory at the horizon.
        // Position test in the opening 70.9 % -> 74.6 % (z +3.3), self-play even
        // (EXPERIMENTS.md, round 10).
        let mut d = Weights {
            length_half: 150.0,
            length_min: 0.3,
            area_open: 0.0,
            crowd_lo: 8.0,
            crowd_hi: 24.0,
            ..w
        };
        if duel_game(st) {
            // Games that start as a duel: the appetite fades between turn 100 and 200 and the
            // length terms fade to 10 %. The top duel snakes stop going for food after turn
            // 100 and are five cells shorter than we were at turn 300; we lost late and with
            // a length lead. Position test from turn 250 on 62.0 % -> 65.3 % (z +8.9),
            // self-play +84 Elo over 1600 games (EXPERIMENTS.md, round 11). Not measured for
            // the last two snakes of a standard game: unchanged there.
            d.appetite_lo = 100.0;
            d.appetite_hi = 200.0;
            d.length_min = 0.1;
        }
        return d;
    }
    // multi-player: being pinned to the wall / boxed in by bigger snakes is the main way we die
    Weights {
        edge: 3.0,
        trap: 12.0,
        ..w
    }
}

pub fn config_for(st: &State, parallel: bool) -> SearchConfig {
    // The ply limit with veto (deep_ply 2, deep_min_opps 3, veto_frac 0.35) moved our standard
    // openings off the walls, but showed no gain in self-play (EXPERIMENTS.md, round 7): off.
    //
    // Standard with two or more opponents alive: focus search (one search per opponent instead
    // of a coalition of all) with the heuristic move for the opponents that do not choose
    // freely. Position test 62.4 % -> 64.7 % (z +3.5), self-play about +50 Elo over seven runs
    // (EXPERIMENTS.md, round 8). Not measured in royale and constrictor: off there.
    //
    // focus_reach 1 in standard: besides the focus opponent only opponents whose head is next
    // to ours choose freely, so we keep less distance to heads that cannot reach us. Position
    // test 63.2 % -> 65.3 % (z +4.6), self-play even (EXPERIMENTS.md, round 10).
    let standard = mode_of(st) == "standard";
    let d = SearchConfig::default();
    SearchConfig {
        weights: weights_for(st),
        parallel,
        focus: standard,
        smart_default: standard,
        focus_reach: if standard { 1 } else { d.focus_reach },
        ..d
    }
}

/// A game that started as a duel, as opposed to the last two snakes of a bigger game. Without
/// the number of snakes at the start (`Rules::players`) the answer is no.
pub fn duel_game(st: &State) -> bool {
    st.rules.players == 2
}

/// The mode a position belongs to, as the keys of a variant: "constrictor", "royale", "duel"
/// (at most one opponent alive) or "standard".
pub fn mode_of(st: &State) -> &'static str {
    if st.rules.constrictor {
        "constrictor"
    } else if st.rules.royale {
        "royale"
    } else if st.alive_opponents() <= 1 {
        "duel"
    } else {
        "standard"
    }
}

/// Apply a variant on top of the production config. A variant is a JSON object:
///
/// ```json
/// {"weights": {"length": 20}, "focus": true, "duel": {"weights": {"area_open": 0.3}}}
/// ```
///
/// Keys: `weights` (partial [`Weights`]), `focus`, `focus_reach`, `focus_root`, `smart`, `max_reach`,
/// `max_relevant`, `deep_ply`, `deep_reach`, `deep_min_opps`, `veto_frac`. The same keys inside
/// a mode object ("duel", "standard", "royale", "constrictor") only apply in that mode. "duel"
/// covers every position with one opponent left; the keys inside "duel_game" only apply in
/// games that started as a duel ([`duel_game`]).
pub fn apply_variant(cfg: &mut SearchConfig, st: &State, v: &serde_json::Value) {
    apply_keys(cfg, v);
    let mode = mode_of(st);
    if let Some(m) = v.get(mode) {
        apply_keys(cfg, m);
    }
    if mode == "duel" && duel_game(st) {
        if let Some(m) = v.get("duel_game") {
            apply_keys(cfg, m);
        }
    }
}

fn apply_keys(cfg: &mut SearchConfig, v: &serde_json::Value) {
    let Some(o) = v.as_object() else { return };
    if let Some(over) = o.get("weights").and_then(|w| w.as_object()) {
        let mut w = serde_json::to_value(&cfg.weights).unwrap();
        for (k, x) in over {
            w[k] = x.clone();
        }
        cfg.weights = serde_json::from_value(w).expect("bad weights in variant");
    }
    let int = |k: &str| o.get(k).and_then(|x| x.as_i64());
    let flag = |k: &str| o.get(k).map(|x| x.as_bool().unwrap_or(x.as_i64() == Some(1)));
    if let Some(x) = flag("focus") {
        cfg.focus = x;
    }
    if let Some(x) = int("focus_reach") {
        cfg.focus_reach = x as i32;
    }
    if let Some(x) = int("focus_root") {
        cfg.focus_root = x as i32;
    }
    if let Some(x) = flag("smart") {
        cfg.smart_default = x;
    }
    if let Some(x) = int("max_reach") {
        cfg.max_reach = x as i32;
    }
    if let Some(x) = int("max_relevant") {
        cfg.max_relevant = x as usize;
    }
    if let Some(x) = int("deep_ply") {
        cfg.deep_ply = x as usize;
    }
    if let Some(x) = int("deep_reach") {
        cfg.deep_reach = x as i32;
    }
    if let Some(x) = int("deep_min_opps") {
        cfg.deep_min_opps = x as usize;
    }
    if let Some(x) = o.get("veto_frac").and_then(|x| x.as_f64()) {
        cfg.veto_frac = x as f32;
    }
}

/// Variant for the analysis tools from the environment: `VARIANT` holds a variant object (see
/// [`apply_variant`]). The older single switches still work: AGREE_W (partial weights),
/// AGREE_FOCUS, AGREE_FOCUSREACH, AGREE_SMART, AGREE_REACH, AGREE_REL, AGREE_DEEP,
/// AGREE_DEEPREACH, AGREE_DEEPOPPS, AGREE_VETO.
pub fn apply_env(cfg: &mut SearchConfig, st: &State) {
    if let Ok(js) = std::env::var("VARIANT") {
        let v: serde_json::Value = serde_json::from_str(&js).expect("VARIANT must be JSON");
        apply_variant(cfg, st, &v);
    }
    let mut v = serde_json::Map::new();
    if let Ok(js) = std::env::var("AGREE_W") {
        v.insert("weights".into(), serde_json::from_str(&js).expect("AGREE_W must be JSON"));
    }
    for (env, key) in [
        ("AGREE_FOCUS", "focus"),
        ("AGREE_FOCUSREACH", "focus_reach"),
        ("AGREE_SMART", "smart"),
        ("AGREE_REACH", "max_reach"),
        ("AGREE_REL", "max_relevant"),
        ("AGREE_DEEP", "deep_ply"),
        ("AGREE_DEEPREACH", "deep_reach"),
        ("AGREE_DEEPOPPS", "deep_min_opps"),
    ] {
        if let Some(x) = std::env::var(env).ok().and_then(|x| x.parse::<i64>().ok()) {
            v.insert(key.into(), x.into());
        }
    }
    if let Some(x) = std::env::var("AGREE_VETO").ok().and_then(|x| x.parse::<f64>().ok()) {
        v.insert("veto_frac".into(), x.into());
    }
    apply_keys(cfg, &serde_json::Value::Object(v));
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::GameState;

    fn state(map: &str, hazards: &str) -> State {
        let js = std::fs::read_to_string("tests/fixtures/standard.json").unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&js).unwrap();
        v["game"]["map"] = map.into();
        v["board"]["hazards"] = serde_json::from_str(hazards).unwrap();
        serde_json::from_value::<GameState>(v).unwrap().to_state()
    }

    #[test]
    fn appetite_fades_only_in_games_that_start_as_a_duel() {
        let js = std::fs::read_to_string("tests/fixtures/duel.json").unwrap();
        let mut gs: GameState = serde_json::from_str(&js).unwrap();
        assert_eq!(gs.board.snakes.len(), 2);
        // the engine does not say how many snakes started: the old duel profile
        let end_of_standard = profile(&gs.to_state());
        assert_eq!((end_of_standard.appetite_hi, end_of_standard.length_min), (0.0, 0.3));
        gs.game.players = 4;
        assert_eq!(profile(&gs.to_state()), end_of_standard);
        gs.game.players = 2;
        let duel = profile(&gs.to_state());
        assert_eq!((duel.appetite_lo, duel.appetite_hi, duel.length_min), (100.0, 200.0, 0.1));
        assert_eq!(duel.area_open, end_of_standard.area_open);
    }

    #[test]
    fn royale_profile_needs_map_or_hazards() {
        let base = Weights::default().hunger_health;
        assert_eq!(profile(&state("standard", "[]")).hunger_health, base);
        assert_eq!(profile(&state("royale", "[]")).hunger_health, 90);
        assert_eq!(profile(&state("standard", r#"[{"x":0,"y":0}]"#)).hunger_health, 90);
    }
}
