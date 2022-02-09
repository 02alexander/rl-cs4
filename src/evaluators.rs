
use crate::connect4::{Connect4, Player, Action, GameState, TileStates};

pub trait Evaluator {
    fn value(&self, board: &Connect4) -> f64;
}

#[derive(Clone, Copy)]
pub struct SimpleEval {
    player: Player
}

impl SimpleEval {
    pub fn new(player: Player) -> SimpleEval {
        SimpleEval {player}
    }
}

impl Evaluator for SimpleEval {
    fn value(&self, board: &Connect4) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                0.0
            },
        }
    }    
}

pub struct LinesEval {
    player: Player
}

impl LinesEval {

    fn line_value(v: &Vec<TileStates>) -> f64 {
        for i in 0..v.len() {

        }
        unimplemented!()
    }

}

