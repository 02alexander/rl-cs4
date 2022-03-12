
use std::fmt;
use std::ops;
use serde::{Serialize, Deserialize};

pub const BOARD_WIDTH: usize = 7;
pub const BOARD_HEIGHT: usize = 6;
pub const N_ACTIONS: usize = BOARD_WIDTH;

pub const REWARD_LOSE: f64 = -1.0;
pub const REWARD_WIN: f64 = 1.0;
pub const REWARD_DRAW: f64 = 0.0;

pub type Action = usize; // a value in the range of [0,BOARD_WIDTH)

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum TileStates {
    Empty,
    Full(Player),
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Player {
    Red=1,
    Yellow=2,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameState {
    Won(Player),
    Draw,
    InProgress
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Connect4 {
    //pub board: Vec<Vec<TileStates>>,
    //pub board: Vec<TileStates>,

    // tile on board takes up 2 bits, 0 for empty, 1 for red, 2 for yellow. 
    // starts in bottom left corner and goes row by row.
    pub board: u128,
    pub cur_player: Player,
    pub game_state: GameState,
}

impl ops::Not for Player {
    type Output = Player;
    fn not(self) -> Self {
        match self {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
        }
    }
}

impl Connect4 {
    pub fn new() -> Self {
        Connect4 {
            //board: vec![vec![TileStates::Empty; BOARD_HEIGHT];BOARD_WIDTH], // board[x][y] where x,y=(0,0) is the bottom left corner.
            //board: vec![TileStates::Empty; BOARD_HEIGHT*BOARD_WIDTH],
            board: 0,
            cur_player: Player::Red,
            game_state: GameState::InProgress,
        }
    }

    pub fn play_move(&mut self, action: Action) {
        assert_eq!(self.game_state, GameState::InProgress);
        if !self.is_valid_move(action) {
            return
        }
        let ap = self.action_pos(action);
        self.set(ap[0],ap[1], self.cur_player as u8);
        if self.player_won(ap) {
            self.game_state = GameState::Won(self.cur_player);
        } else if self.is_full() {
            self.game_state = GameState::Draw;
        } else {
            self.game_state = GameState::InProgress;
        }
        self.cur_player = !self.cur_player;
    }

    pub fn reverse_last_action(&mut self, last_action: Action) {
        let ap = self.pos_from_action(last_action);
        self.set(ap[0],ap[1],0);
        self.game_state = GameState::InProgress;
        self.cur_player = !self.cur_player;
    }

    pub fn player_won(&self, piece_pos: [usize; 2]) -> bool {
        let directions: [[i32;2];4] = [[1,0],[0,1],[-1,1], [1,1]];
        let player = if let p = self.get(piece_pos[0],piece_pos[1]) {
            p
        } else {
            panic!("player_won() passed piece_pos with empty square");
        };
        for direction in directions {
            let mut sm = 1;
            for i in 1..4 {
                let curx = direction[0]*i+piece_pos[0] as i32;
                let cury = direction[1]*i+piece_pos[1] as i32;
                if !self.in_board(curx, cury) {
                    break;
                } else if player as u8 != self.get(curx as usize,cury as usize) {
                    break;
                } 
                sm += 1;
            }
            for i in 1..4 {
                let i = -i;
                let curx = direction[0]*i+piece_pos[0] as i32;
                let cury = direction[1]*i+piece_pos[1] as i32;
                if !self.in_board(curx, cury) {
                    break;
                } else if player as u8 != self.get(curx as usize,cury as usize) {
                    break;
                }
                sm += 1;
            }
            if sm >= 4 {
                return true;
            }
        }
        false
    }

    pub fn in_board(&self, x:i32,y:i32) -> bool {
        x >= 0 && y >= 0 && x < BOARD_WIDTH as i32 && y < BOARD_HEIGHT as i32 
    }

    // Returns where piece will be placed if 'action' is played.
    fn action_pos(&self, action: Action) -> [usize; 2] {
        for cur_y in 0..BOARD_HEIGHT {
            if  self.get(action,cur_y) == 0 {
                return [action, cur_y];
            }
        }
        unimplemented!()
    }

    // Returns where piece placed from last played action 'action'
    fn pos_from_action(&self, action: Action) -> [usize; 2] {
        for cur_y in 1..BOARD_HEIGHT {
            if  self.get(action,cur_y) == 0 {
                return [action, cur_y-1];
            }
        }
        [action, BOARD_HEIGHT-1] 
    }

    pub fn is_full(&self) -> bool {
        !(0..N_ACTIONS).any(|action|self.is_valid_move(action))
    }

    pub fn is_valid_move(&self, action: Action) -> bool {
        assert!(action < BOARD_WIDTH);
        self.get(action,BOARD_HEIGHT-1) == 0
    }

    pub fn valid_moves(&self) -> Vec<Action> {

        // it's 2.5 time faster to use Vec::with_capacity than Vec::new
        let mut v = Vec::with_capacity(BOARD_WIDTH); 
        for i in 0..BOARD_WIDTH {
            if self.is_valid_move(i) {
                v.push(i);
            }
        }
        v
    }

    // mirrors board around the middle of the board.
    // assumes BOARD_WIDTH = 7. so if BOARD_WIDTH changes then so must this function
    pub fn symmetry(&self) -> Connect4 {
        //let col_mask: u128 = (((((((((3<<BOARD_WIDTH*2)+3)<<BOARD_WIDTH*2)+3)<<BOARD_WIDTH*2)+3)<<BOARD_WIDTH*2)+3)<<BOARD_WIDTH*2)+3;
        let mut col_mask: u128 = 0;
        for _ in 0..BOARD_HEIGHT {
            col_mask = (col_mask<<BOARD_WIDTH*2)+3;
        }
        let mut new_board:u128 = 0;
        for i in 0..3 {
            new_board += (self.board&(col_mask<<(i*2)))<<((3-i)*4);
        }
        for i in 0..3 {
            new_board += (self.board&(col_mask<<(4+i)*2))>>(i+1)*4;
        }
        new_board += self.board&(col_mask<<3*2);
        
        Connect4 {
            board: new_board,
            cur_player: self.cur_player,
            game_state: self.game_state,
        }
    }

    pub fn vectorize(&self, player: Player) -> Vec<f64> {
        let mut v = Vec::with_capacity(BOARD_WIDTH*BOARD_HEIGHT);
        let mut board = self.board;
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let cur = board as u8 & 3;
                if cur == player as u8 {
                    v.push(1.0);
                } else if cur == !player as u8 {
                    v.push(-1.0);
                } else {
                    v.push(0.0);
                }
                board >>= 2;
                /*if self.get(x,y) == player as u8 {
                    v.push(1.0);
                } else if self.get(x,y) == !player as u8 {
                    v.push(-1.0);
                } else {
                    v.push(0.0);
                }*/
            }
        }
        v
    }

    pub fn set(&mut self, x: usize, y: usize, v: u8) {

        /*let m = match v {
            TileStates::Empty => 0,
            TileStates::Full(Player::Red) => 1,
            TileStates::Full(Player::Yellow) => 2
        };*/
        let k = (2*(x+y*BOARD_WIDTH));
        let mask = 3 << k;
        self.board = (self.board & (!mask)) + ((v as u128) << k);
        //let prev_m = 3 & (self.board >> (2*(x+y*BOARD_WIDTH)));
        //self.board = self.board ^ ( self.board >> (2*(x+y*BOARD_WIDTH)) );
    }

    pub fn get(&self, x: usize, y:usize) -> u8 {
        3 & (self.board >> (2*(x+y*BOARD_WIDTH))) as u8
    }
}

/*
impl std::ops::Index<[usize;2]> for Connect4 {
    type Output = TileStates;
    // idx: [x,y]
    fn index(&self, idx: [usize;2]) -> &Self::Output {
        //&self.board[idx[0]][idx[1]]

        &self.board[ idx[0]+idx[1]*BOARD_WIDTH]
    }
}

impl std::ops::IndexMut<[usize;2]> for Connect4 {
    fn index_mut(&mut self, idx: [usize;2]) -> &mut Self::Output {
        //&mut self.board[idx[0]][idx[1]]
        &mut self.board[ idx[0]+idx[1]*BOARD_WIDTH]
    }
}*/

impl fmt::Debug for Connect4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for y in (0..BOARD_HEIGHT).rev() {
            for x in 0..BOARD_WIDTH {
                match self.get(x,y) {
                    0 => s.push_str("# "),
                    1 => {
                        s.push_str("\x1b[30;41m \x1b[0m ");
                    }
                    _ => {
                        s.push_str("\x1b[30;43m \x1b[0m ");
                    }
                }
            }
            s.push('\n');
        }
        write!(f, "{}", &s)
    }
}

#[cfg(test)] 
mod test {
    use super::*;
    #[test]
    fn player_won() {
        let mut board = Connect4::new();
        let moves = vec![1,2,2,3,2,4,2,5];
        for mv in &moves[0..moves.len()-1] {
            board.play_move(*mv);
            println!("{:?}", board);
            assert_eq!(board.game_state, GameState::InProgress);
        }
        board.play_move(moves[moves.len()-1]);
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        assert_eq!(board.game_state, GameState::Won(Player::Yellow));
    }

    #[test]
    fn reverse_move() {
        let mut board = Connect4::new();
        let moves = vec![1,2,2,3,2,4,2,5];
        for mv in &moves[0..moves.len()-1] {
            board.play_move(*mv);
            println!("{:?}", board);
            assert_eq!(board.game_state, GameState::InProgress);
        }
        board.play_move(moves[moves.len()-1]);
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        assert_eq!(board.game_state, GameState::Won(Player::Yellow));
        board.reverse_last_action(moves[moves.len()-1]);
        assert_eq!(board.game_state, GameState::InProgress);
        board.play_move(6);
        board.play_move(2);
        assert_eq!(board.game_state, GameState::Won(Player::Red));

        let mut board = Connect4::new();
        board.play_move(0);
        board.play_move(0);
        board.play_move(0);
        board.play_move(0);
        board.play_move(0);
        let old_board = board.clone();
        board.play_move(0);
        board.reverse_last_action(0);
        println!("{:?}\n{:?}",old_board,board);
        assert_eq!(old_board.board, board.board);
    }
}


/*
mod bench_indexing {
    use crate::test::{black_box, Bencher};
    use super::*;

    #[bench]
    fn bench_indexing_vec_of_vec(b: &mut Bencher) {
        let v: Vec<Vec<i32>> = vec![ vec![1, 2,3],vec![0,1,2],vec![1,3,4]];
        let x = black_box(1);
        let y = black_box(1);
        b.iter(|| {
            (0..100).fold(0, |_,_| black_box(v[x][y]))
        });
    }

    #[bench]
    fn bench_indexing_long_vec(b: &mut Bencher) {
        let v: Vec<i32> = vec![1,2,3,0,1,2,1,3,4];
        let x = black_box(1);
        let y = black_box(1);
        b.iter(|| {
            (0..100).fold(0, |_,_| black_box(v[x+3*y]))
            //(0..1000).fold(0, |_,_| black_box(0))
        });
    }
}

mod bench_connect4 {
    use crate::test::{black_box, Bencher};
    use super::*;
    #[bench]
    fn in_board(b: &mut Bencher) {
        let mut board = Connect4::new();
        let x = black_box(3);
        let y = black_box(4);
        b.iter(|| {
            (0..100).fold(false, |_,_| black_box(board.in_board(x, y)))
        });
    }
    #[bench]
    fn valid_moves(b: &mut Bencher) {
        let mut board = Connect4::new();
        let x = black_box(3);
        let y = black_box(4);
        b.iter(|| {
            (0..100).fold(Vec::new(), |_,_| black_box(board.valid_moves()))
        });
    }
}*/