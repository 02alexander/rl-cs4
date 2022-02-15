
use crate::connect4::{Connect4, Player, Action, GameState, TileStates};
use crate::connect4;

pub trait Evaluator {
    fn value(&self, board: &Connect4) -> f64;
    fn set_player(&mut self, player: Player);
    fn get_player(&self) -> Player;
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
                //if p == self.player {1./0.} else {-1./0.}
                if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                0.0
            },
        }
    }
    fn set_player(&mut self, player: Player) {
        self.player = player
    
    } 
    fn get_player(&self) -> Player {
        self.player
    } 
}

pub struct LinesEval {
    pub params: Vec<f64>,
    pub player: Player
}

impl Evaluator for LinesEval {
    fn value(&self, board: &Connect4) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == self.player {100000.0} else {-100000.0}
                //if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                self.lines_evaluation(board)
            },
        }
    }
    fn set_player(&mut self, player: Player) {
        self.player = player
    } 
    fn get_player(&self) -> Player {
        self.player
    } 
}

impl LinesEval {

    pub fn new(player: Player) -> LinesEval {
        LinesEval {
            params: vec![0.0, 0.0], // [ v for 2 in a row, v for 3 in a row]
            player,
        }
    }

    fn lines_evaluation(&self, board: &Connect4) -> f64 {
        let mut total = 0.0;
        let directions = [[1, 0],[0, 1], [1,1]];
        //iterates over the first row and the first column.
        for (r,c) in (0..connect4::BOARD_HEIGHT).map(|r|(r,0)).chain((0..connect4::BOARD_HEIGHT).map(|c|(0,c))) {
            //println!("{:?}", (r,c));
            for dir in directions {
                let mut line = Vec::with_capacity(connect4::BOARD_WIDTH); 
                let mut k = 0;
                loop {
                    if !board.in_board(k*dir[1]+c as i32, k*dir[0]+r as i32) {
                        break
                    }
                    line.push(board[[(k*dir[1]+c as i32) as usize, (k*dir[0]+r as i32) as usize]]);
                    k += 1;
                }
                //println!("line={:?}", line);
                total += self.line_value(&line);
            }
        }
        total
    }

    fn line_value(&self, v: &Vec<TileStates>) -> f64 {
        let mut last_opponent: i32 = -1;
        let mut count: u32 = 0;
        let mut totv = 0.0;
        //println!("{:?}", v);
        for i in 0..v.len() {
            match v[i] {
                TileStates::Empty => {
                    
                },
                TileStates::Full(p) => {
                    if p != self.player {
                        last_opponent = i as i32;
                        count = 0;
                    } else {
                        //println!("added to count");
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

