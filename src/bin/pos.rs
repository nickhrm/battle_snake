//! Engine request of one snake at one turn of a stored replay, for `bench` and test fixtures.
//!
//! pos <snake name> <game.json> <turn> > position.json

use honey_badger::replay::Replay;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rp = Replay::load(&args[1]).expect("replay");
    let turn: u64 = args[2].parse().expect("turn");
    let i = rp.frames.iter().position(|f| f["Turn"].as_u64() == Some(turn)).expect("turn not in game");
    let id = rp.find(i, &args[0]).expect("snake not in game")["ID"].clone();
    let gs = rp.request(i, &id).expect("snake is dead at this turn");
    println!("{}", serde_json::to_string(&gs).unwrap());
}
