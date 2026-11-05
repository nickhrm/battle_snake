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
        ..SearchConfig::default()
    }
}
