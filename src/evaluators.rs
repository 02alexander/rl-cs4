
use crate::connect4::{Connect4, Player, Action, GameState, TileStates};

pub trait Evaluator {
    fn value(&self, board: &Connect4) -> f64;
}


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
                if p == self.player {1.0} else {-1.0}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                0.0
                /*let mut sm: f64 = 0.0;
                for x in 0..3 {
                    for y in 0..3 {
                        if board.in_board(x,y) && board[[x as usize,y as usize]] != TileStates::Empty {
                            sm += 1.0;
                        }
                    }
                }
                sm.exp()/(1.0+sm.exp())
                */
            },
        }
    }
}


/*pub struct LinesEval {
    player: Player
}*/

