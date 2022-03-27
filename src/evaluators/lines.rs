
use serde::{Serialize, Deserialize};
use crate::games::{Player, GameState, TileStates};
use crate::games::connect4::{Connect4, BOARD_HEIGHT, BOARD_WIDTH};
use super::Evaluator;

#[derive(Clone, Serialize, Deserialize)]
pub struct LinesEval {
    pub params: Vec<f64>,
}

impl Evaluator<Connect4> for LinesEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                self.lines_evaluation(board, player)
            },
        }
    }
    fn gradient(&self, _board: &Connect4, _player: Player) -> Vec<f64> {
        unimplemented!()
    }
    fn apply_update(&mut self, _update: &[f64]) {
        unimplemented!()
    }
    fn get_params(&self) -> Vec<f64> {
        self.params.clone()
    }
}

impl LinesEval {

    pub fn new() -> LinesEval {
        LinesEval {
            params: vec![0.0, 0.0], // [ v for 2 in a row, v for 3 in a row]
        }
    }

    fn lines_evaluation(&self, board: &Connect4, player: Player) -> f64 {
        let mut total = 0.0;
        let directions = [[1, 0],[0, 1], [1,1]];
        //iterates over the first row and the first column.
        for (r,c) in (0..BOARD_HEIGHT).map(|r|(r,0)).chain((0..BOARD_HEIGHT).map(|c|(0,c))) {
            for dir in directions {
                let mut line = Vec::with_capacity(BOARD_WIDTH); 
                let mut k = 0;
                loop {
                    if !board.in_board(k*dir[1]+c as i32, k*dir[0]+r as i32) {
                        break
                    }
                    line.push(match board.get((k*dir[1]+c as i32) as usize, (k*dir[0]+r as i32) as usize) {
                        0 => TileStates::Empty,
                        1 => TileStates::Full(Player::Red),
                        _ => TileStates::Full(Player::Yellow)
                    });
                    k += 1;
                }
                total += self.line_value(&line, player);
            }
        }
        total
    }

    fn line_value(&self, v: &Vec<TileStates>, player: Player) -> f64 {
        let mut last_opponent: i32 = -1;
        let mut count: u32 = 0;
        let mut totv = 0.0;
        for i in 0..v.len() {
            match v[i] {
                TileStates::Empty => {
                    
                },
                TileStates::Full(p) => {
                    if p != player {
                        last_opponent = i as i32;
                        count = 0;
                    } else {
                        count += 1;
                    }
                }
            }
            if i as i32-last_opponent >= 4 {
                totv += count as f64;
            }
        }
        totv
    }

}
