
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::games::{Player, GameState};
use crate::games::Game;
use crate::matchmaker::PlayableGame;
use std::io;
use std::io::BufRead;

pub const BOARD_WIDTH: usize = 7;
pub const BOARD_HEIGHT: usize = 6;
pub const N_ACTIONS: usize = BOARD_WIDTH;

pub const REWARD_LOSE: f64 = -1.0;
pub const REWARD_WIN: f64 = 1.0;
pub const REWARD_DRAW: f64 = 0.0;

pub type Action = usize; // a value in the range of [0,BOARD_WIDTH)


#[derive(Clone, Serialize, Deserialize)]
pub struct Connect4 {
    // tile on board takes up 2 bits, 0 for empty, 1 for red, 2 for yellow. 
    // starts in bottom left corner and goes row by row.
    pub board: u128,
    pub cur_player: Player,
    pub game_state: GameState,
}



impl Connect4 {

    pub fn player_won(&self, piece_pos: [usize; 2]) -> bool {
        let directions: [[i32;2];4] = [[1,0],[0,1],[-1,1], [1,1]];
        let player = self.get(piece_pos[0],piece_pos[1]);
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

    pub fn set(&mut self, x: usize, y: usize, v: u8) {
        let k = 2*(x+y*BOARD_WIDTH);
        let mask = 3 << k;
        self.board = (self.board & (!mask)) + ((v as u128) << k);
    }

    pub fn get(&self, x: usize, y:usize) -> u8 {
        3 & (self.board >> (2*(x+y*BOARD_WIDTH))) as u8
    }
}

impl Game for Connect4 {
    type Action = usize;

    fn new() -> Self {
        Connect4 {
            board: 0,
            cur_player: Player::Red,
            game_state: GameState::InProgress,
        }
    }

    // Plays action for player self.cur_player
    fn play_action(&mut self, action: Action) {
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

    // Reverses last action if the last action is last_action.
    fn reverse_last_action(&mut self, last_action: Action) {
        let ap = self.pos_from_action(last_action);
        self.set(ap[0],ap[1],0);
        self.game_state = GameState::InProgress;
        self.cur_player = !self.cur_player;
    }

    fn legal_actions(&self) -> Box<dyn Iterator<Item=Action>> {

        // it's 2.5 time faster to use Vec::with_capacity than Vec::new
        let mut v = Vec::with_capacity(BOARD_WIDTH); 
        for i in 0..BOARD_WIDTH {
            if self.is_valid_move(i) {
                v.push(i);
            }
        }
        Box::new(v.into_iter())
    }

    fn game_state(&self) -> GameState {
        self.game_state
    }

    fn cur_player(&self) -> Player {
        self.cur_player
    }

    fn vectorize(&self, player: Player) -> Vec<f64> {
        let mut v = Vec::with_capacity(BOARD_WIDTH*BOARD_HEIGHT);
        let mut board = self.board;
        for _ in 0..BOARD_WIDTH {
            for _ in 0..BOARD_HEIGHT {
                let cur = board as u8 & 3;
                if cur == player as u8 {
                    v.push(1.0);
                } else if cur == !player as u8 {
                    v.push(-1.0);
                } else {
                    v.push(0.0);
                }
                board >>= 2;
            }
        }
        v
    }
    fn uid(&self) -> u128 {
        self.board
    }
}


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

impl PlayableGame for Connect4 {
    // returns (action, is_reverse)
    fn get_action_from_user(&self) -> (Action, bool) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if line.as_bytes()[0] == 'z' as u8 {
                return (0, true);
            } else if let Ok(a) = line.parse::<usize>() {
                if a < BOARD_WIDTH {
                    if !self.is_valid_move(a) {
                        println!("Column alread full");
                        continue;
                    }
                    return (a, false);
                } else {
                    println!("Not in range 0..{}", BOARD_WIDTH);    
                }
            } else {
                println!("Invalid input: try again");
            }
        }
        panic!("Failed to get input from user");
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
            board.play_action(*mv);
            println!("{:?}", board);
            assert_eq!(board.game_state, GameState::InProgress);
        }
        board.play_action(moves[moves.len()-1]);
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        assert_eq!(board.game_state, GameState::Won(Player::Yellow));
    }

    #[test]
    fn reverse_move() {
        let mut board = Connect4::new();
        let moves = vec![1,2,2,3,2,4,2,5];
        for mv in &moves[0..moves.len()-1] {
            board.play_action(*mv);
            println!("{:?}", board);
            assert_eq!(board.game_state, GameState::InProgress);
        }
        board.play_action(moves[moves.len()-1]);
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        assert_eq!(board.game_state, GameState::Won(Player::Yellow));
        board.reverse_last_action(moves[moves.len()-1]);
        assert_eq!(board.game_state, GameState::InProgress);
        board.play_action(6);
        board.play_action(2);
        assert_eq!(board.game_state, GameState::Won(Player::Red));

        let mut board = Connect4::new();
        board.play_action(0);
        board.play_action(0);
        board.play_action(0);
        board.play_action(0);
        board.play_action(0);
        let old_board = board.clone();
        board.play_action(0);
        board.reverse_last_action(0);
        println!("{:?}\n{:?}",old_board,board);
        assert_eq!(old_board.board, board.board);
    }
}