
use serde::{Serialize, Deserialize};
use crate::games::{Player, GameState, TileStates};
use crate::games::Game;
use std::fmt;
use std::io::BufRead;

const BOARD_SIZE: usize = 8;

type Action = (usize, usize);

#[derive(Clone, Serialize, Deserialize)]
pub struct Stack4 {
    // tile on board takes up 2 bits, 0 for empty, 1 for red, 2 for yellow. 
    // starts in bottom left corner and goes row by row.
    pub board: u128,
    pub cur_player: Player,
    pub game_state: GameState,
}

impl Stack4 {
    pub fn new() -> Self {
        Self {
            board: 0,
            cur_player: Player::Red,
            game_state: GameState::InProgress
        }
    }

    pub fn player_won(&self, piece_pos: [usize; 2]) -> bool {
        let directions: [[i32;2];4] = [[1,0],[0,1],[-1,1], [1,1]];
        let player = self.get(piece_pos[0],piece_pos[1]);
        for direction in directions {
            let mut sm = 1;
            for i in 1..4 {
                let curx = direction[0]*i+piece_pos[0] as i32;
                let cury = direction[1]*i+piece_pos[1] as i32;
                if !Stack4::in_board(curx, cury) {
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
                if !Stack4::in_board(curx, cury) {
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

    fn is_full(&self) -> bool {
        self.legal_actions().len() == 0
    }

    fn in_board(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < BOARD_SIZE as i32 && y < BOARD_SIZE as i32
    }

    pub fn set(&mut self, x: usize, y: usize, v: u8) {
        let k = 2*(x+y*BOARD_SIZE);
        let mask = 3 << k;
        self.board = (self.board & (!mask)) + ((v as u128) << k);
    }

    pub fn get(&self, x: usize, y:usize) -> u8 {
        3 & (self.board >> (2*(x+y*BOARD_SIZE))) as u8
    }

    // returns (action, is_reverse)
    fn get_move_from_user(board: &Stack4) -> (Action, bool) {
        let stdin = std::io::stdin();
        let legal_actions = board.legal_actions();

        fn parse_cord(s: &str) -> Option<(usize,usize)> {
            let mut numbers = s.split(',');
            let x = numbers.next()?.parse::<usize>().ok()?;
            let y = numbers.next()?.parse::<usize>().ok()?;
            Some((x,y))
        }

        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if line.as_bytes()[0] == 'z' as u8 {
                return ((0,0), true);
            } else if let Some((x,y)) = parse_cord(&line) {
                if x < BOARD_SIZE && y < BOARD_SIZE {
                    if !legal_actions.iter().any(|c|*c==(x,y)) {
                        println!("Illegal action");
                        continue;
                    }
                    return ((x,y), false);
                } else {
                    println!("Not in range (0..{}, 0..{})", BOARD_SIZE, BOARD_SIZE);    
                }
            } else {
                println!("Invalid input: try again");
            }
        }
        panic!("Failed to get input from user");
    }
    
    pub fn user_vs_user() {
        let mut board = Stack4::new();
        let mut last_action = (0,0);
        loop {
            println!("{:?}", board);
            println!("{:?}", board.game_state);
            let (action, reverse) = Stack4::get_move_from_user(&board);
            if reverse {
                board.reverse_last_action(last_action);
            } else {
                last_action = action;
                board.play_action(action);
                match board.game_state {
                    GameState::Draw => {
                        println!("Draw");
                    }
                    GameState::InProgress => {},
                    GameState::Won(player) => {
                        println!("{:?} won", player);   
                    }
                }
            }
        }
    }
}

impl Game for Stack4 {
    type Action = (usize, usize); // x,y coordinates of the placed piece.

    // Assumes that 'action' is a legal action.
    fn play_action(&mut self, action: Self::Action) {
        assert_eq!(self.game_state, GameState::InProgress);
        self.set(action.0, action.1, self.cur_player as u8);

        if self.player_won([action.0, action.1]) {
            self.game_state = GameState::Won(self.cur_player);
        } else if self.is_full() {
            self.game_state = GameState::Draw;
        } else {
            self.game_state = GameState::InProgress;
        }
        self.cur_player = !self.cur_player;
    }

    fn reverse_last_action(&mut self, last_action: Self::Action) {
        self.set(last_action.0, last_action.1, 0);
    }

    fn game_state(&self) -> GameState {
        self.game_state
    }

    fn cur_player(&self) -> Player {
        self.cur_player
    }

    fn legal_actions(&self) -> Vec<Self::Action> {
        let dirs = [[1,0], [0,1], [-1,0], [0, -1]];
        let starts = [[0,0], [BOARD_SIZE-1, 0], [BOARD_SIZE-1, BOARD_SIZE-1], [0, BOARD_SIZE-1]];
        let mut actions = Vec::with_capacity(BOARD_SIZE*4);
        for (dir, start) in dirs.iter().zip(starts) {
            let inward_direction = [-dir[1], dir[0]];
            for c in 0..BOARD_SIZE as i32 {
                let cur_start = [start[0] as i32+dir[0] as i32*c, start[1] as i32+dir[1] as i32*c];
                for k in 0..BOARD_SIZE {
                    let cur_cord = [ 
                        (cur_start[0]+k as i32*inward_direction[0]) as usize, 
                        (cur_start[1]+k as i32*inward_direction[1]) as usize
                    ];
                    // 0 represents TileStates::Empty
                    if self.get(cur_cord[0], cur_cord[1]) == 0 {
                        actions.push((cur_cord[0], cur_cord[1]));
                        break
                    }
                }
            }
        }
        let set: std::collections::HashSet<_> = actions.drain(..).collect();
        actions.extend(set);
        println!("{}", actions.len());
        actions
    }

    fn vectorize(&self, player: Player) -> Vec<f64> {
        let mut v = Vec::with_capacity(BOARD_SIZE*BOARD_SIZE);
        let mut board = self.board;
        for _ in 0..BOARD_SIZE {
            for _ in 0..BOARD_SIZE {
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

impl fmt::Debug for Stack4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        let legal_actions = self.legal_actions();
        for y in (0..BOARD_SIZE).rev() {
            for x in 0..BOARD_SIZE {
                match self.get(x,y) {
                    0 => {
                        if legal_actions.iter().any(|c| *c==(x,y)) {
                            s.push_str("O ")
                        } else {
                            s.push_str("# ") 
                        }
                    },
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
