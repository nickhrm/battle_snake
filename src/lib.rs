#![allow(clippy::needless_range_loop)]
pub mod api;
pub mod eval;
pub mod game;
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
        // duel
        return w;
    }
    // multi-player: being pinned to the wall / boxed in by bigger snakes is the main way we die
    Weights {
        edge: 3.0,
        trap: 12.0,
        ..w
    }
}

pub fn config_for(st: &State, parallel: bool) -> SearchConfig {
    SearchConfig {
        weights: weights_for(st),
        parallel,
        pv_split: std::env::var("PV_SPLIT").map(|v| v != "0").unwrap_or(true),
        ..SearchConfig::default()
    }
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
    fn royale_profile_needs_map_or_hazards() {
        let base = Weights::default().hunger_health;
        assert_eq!(profile(&state("standard", "[]")).hunger_health, base);
        assert_eq!(profile(&state("royale", "[]")).hunger_health, 90);
        assert_eq!(profile(&state("standard", r#"[{"x":0,"y":0}]"#)).hunger_health, 90);
    }
}
