use super::Evaluator;
use crate::games::connect4::{Connect4, BOARD_HEIGHT, BOARD_WIDTH};
use crate::games::stack4::Stack4;
use crate::games::{GameState, Player};
use serde::{Deserialize, Serialize};

fn pieces_in_row(board: &Connect4, pos: [usize; 2], dir: [i32; 2], player: Player) -> u32 {
    let mut k = 1;
    while board.in_board(
        pos[0] as i32 + dir[0] as i32 * k,
        pos[1] as i32 + dir[1] as i32 * k,
    ) && board.get(
        (pos[0] as i32 + dir[0] * k) as usize,
        (pos[1] as i32 + dir[1] * k) as usize,
    ) == player as u8
    {
        k += 1;
    }
    k as u32 - 1
}

fn pieces_in_row_stack4(board: &Stack4, pos: [usize; 2], dir: [i32; 2], player: Player) -> u32 {
    let mut k = 1;
    while Stack4::in_board(
        pos[0] as i32 + dir[0] as i32 * k,
        pos[1] as i32 + dir[1] as i32 * k,
    ) && board.get(
        (pos[0] as i32 + dir[0] * k) as usize,
        (pos[1] as i32 + dir[1] * k) as usize,
    ) == player as u8
    {
        k += 1;
    }
    k as u32 - 1
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConsequtiveEval {
    pub params: Vec<f64>,
}

impl Evaluator<Connect4> for ConsequtiveEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == player {
                    1. / 0.
                } else {
                    -1. / 0.
                }
            }
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let features = self.features(board, player);
                let mut tot = 0.0;
                for (f, v) in features.iter().zip(self.params.iter()) {
                    tot += *f as f64 * v;
                }
                tot
            }
        }
    }
    fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        self.features(board, player)
    }
    fn apply_update(&mut self, update: &[f64]) {
        for (p, d) in self.params.iter_mut().zip(update) {
            *p += d;
        }
    }
    fn get_params(&self) -> Vec<f64> {
        self.params.clone()
    }
}

impl Evaluator<Stack4> for ConsequtiveEval {
    fn value(&self, board: &Stack4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == player {
                    1. / 0.
                } else {
                    -1. / 0.
                }
            }
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let features = self.features_stack4(board, player);
                let mut tot = 0.0;
                for (f, v) in features.iter().zip(self.params.iter()) {
                    tot += *f as f64 * v;
                }
                tot
            }
        }
    }
    fn gradient(&self, board: &Stack4, player: Player) -> Vec<f64> {
        self.features_stack4(board, player)
    }
    fn apply_update(&mut self, update: &[f64]) {
        for (p, d) in self.params.iter_mut().zip(update) {
            *p += d;
        }
    }
    fn get_params(&self) -> Vec<f64> {
        self.params.clone()
    }
}

impl ConsequtiveEval {
    pub fn new() -> Self {
        ConsequtiveEval {
            params: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // [v for 1 in a row,  v for 2 in a row, v for 3 in a row,    for opponent]
        }
    }

    fn features(&self, board: &Connect4, player: Player) -> Vec<f64> {
        let directions = [[1, 0], [1, 1], [0, 1], [-1, 1]];
        let mut f = vec![0; 6];
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if board.get(x, y) != 0 {
                    continue;
                }
                for dir in directions {
                    let a = pieces_in_row(board, [x, y], dir, player);
                    let b = pieces_in_row(board, [x, y], [-dir[0], -dir[1]], player);
                    let l = 3.min(a + b);
                    if l >= 1 {
                        f[l as usize - 1] += 1;
                    }
                }
            }
        }
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if board.get(x, y) != 0 {
                    continue;
                }
                for dir in directions {
                    let a = pieces_in_row(board, [x, y], dir, !player);
                    let b = pieces_in_row(board, [x, y], [-dir[0], -dir[1]], !player);
                    let l = 3.min(a + b);
                    if l >= 1 {
                        f[l as usize - 1 + 3] += 1;
                    }
                }
            }
        }
        let mx = 10.0;
        f.iter()
            .map(|x| mx * (1.0 - (-x as f64 / mx).exp()))
            .collect()
    }
    fn features_stack4(&self, board: &Stack4, player: Player) -> Vec<f64> {
        let directions = [[1, 0], [1, 1], [0, 1], [-1, 1]];
        let mut f = vec![0; 6];
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if board.get(x, y) != 0 {
                    continue;
                }
                for dir in directions {
                    let a = pieces_in_row_stack4(board, [x, y], dir, player);
                    let b = pieces_in_row_stack4(board, [x, y], [-dir[0], -dir[1]], player);
                    let l = 3.min(a + b);
                    if l >= 1 {
                        f[l as usize - 1] += 1;
                    }
                }
            }
        }
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if board.get(x, y) != 0 {
                    continue;
                }
                for dir in directions {
                    let a = pieces_in_row_stack4(board, [x, y], dir, !player);
                    let b = pieces_in_row_stack4(board, [x, y], [-dir[0], -dir[1]], !player);
                    let l = 3.min(a + b);
                    if l >= 1 {
                        f[l as usize - 1 + 3] += 1;
                    }
                }
            }
        }
        let mx = 10.0;
        f.iter()
            .map(|x| mx * (1.0 - (-x as f64 / mx).exp()))
            .collect()
    }
}
