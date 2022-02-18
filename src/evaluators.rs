
use crate::connect4::{Connect4, Player, GameState, TileStates};
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

fn pieces_in_row(board: &Connect4, pos: [usize;2], dir: [i32;2], player: Player) -> u32 {
    let mut k = 1;
    while board.in_board(pos[0] as i32+dir[0] as i32*k, pos[1] as i32+dir[1] as i32*k) 
        && board.get((pos[0] as i32+dir[0]*k) as usize, (pos[1] as i32+dir[1]*k) as usize) == player as u8 {
        k += 1;
    }
    k as u32 - 1
}

pub struct LinesEval {
    pub params: Vec<f64>,
    pub player: Player
}

impl Evaluator for LinesEval {
    fn value(&self, board: &Connect4) -> f64 {
        //println!("LinesEval.value()");
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
                    line.push(match board.get((k*dir[1]+c as i32) as usize, (k*dir[0]+r as i32) as usize) {
                        0 => TileStates::Empty,
                        1 => TileStates::Full(Player::Red),
                        _ => TileStates::Full(Player::Yellow)
                    });
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



pub struct ConsequtiveEval {
    pub params: Vec<f64>,
    pub player: Player
}

impl Evaluator for ConsequtiveEval {
    fn value(&self, board: &Connect4) -> f64 {
        //println!("ConsequtiveEval.value()");
        match board.game_state {
            GameState::Won(p) => {
                if p == self.player {10000000.0} else {-10000000.0}
                //if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                //self.lines_evaluation(board)
                let features = self.features(board);
                let mut tot = 0.0;
                for (f, v) in features.iter().zip(self.params.iter()) {
                    tot += *f as f64*v;
                } 
                //println!("{:?}", tot);
                tot
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

impl ConsequtiveEval {

    pub fn new(player: Player) -> Self {
        ConsequtiveEval {
            params: vec![0.0, 10.0, 100.0, 0.0, 0.0, -20.0], // [v for 1 in a row,  v for 2 in a row, v for 3 in a row,    for opponent]
            player,
        }
    }

    fn features(&self, board: &Connect4) -> Vec<u32> {
        let directions = [[1,0], [1,1], [0,1], [-1, 1]];
        let mut f = vec![0;6];
        for x in 0..connect4::BOARD_WIDTH {
            for y in 0..connect4::BOARD_HEIGHT {
                if board.get(x,y) != 0 {
                    continue
                }
                for dir in directions {
                    let a = pieces_in_row(board, [x,y], dir, self.player);
                    let b = pieces_in_row(board, [x,y], [-dir[0], -dir[1]], self.player);
                    let l = 3.min(a+b);
                    /*if l == 3 {
                        println!("{:?} {:?}, {}, {}", [x,y], dir, a,b);
                    }*/
                    if l >= 1 {
                        f[l as usize-1] += 1;
                    }
                }
            }
        }
        for x in 0..connect4::BOARD_WIDTH {
            for y in 0..connect4::BOARD_HEIGHT {
                if board.get(x,y) != 0 {
                    continue
                }
                for dir in directions {
                    let a = pieces_in_row(board, [x,y], dir, !self.player);
                    let b = pieces_in_row(board, [x,y], [-dir[0], -dir[1]], !self.player);
                    let l = 3.min(a+b);
                    if l >= 1 {
                        f[l as usize-1+3] += 1;
                    }
                }
            }
        }
        f
    }
}
