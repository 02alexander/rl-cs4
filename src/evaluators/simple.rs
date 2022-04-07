
use serde::{Serialize, Deserialize};
use crate::games::{Game, Player, GameState};
use super::Evaluator;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SimpleEval {}

impl SimpleEval {
    pub fn new() -> SimpleEval {
        SimpleEval {}
    }
}

impl<T> Evaluator<T> for SimpleEval where T: Game {
    fn value(&self, board: &T, player: Player) -> f64 {
        match board.game_state() {
            GameState::Won(p) => {
                if p == player {1.0/board.length() as f64} else {-1.0/board.length() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                0.0
            },
        }
    }
    fn gradient(&self, _board: &T, _player: Player) -> Vec<f64> {
        unimplemented!()
    }
    fn apply_update(&mut self, _update: &[f64]) {
        unimplemented!()
    }
    fn get_params(&self) -> Vec<f64> {
        unimplemented!()
    }
}