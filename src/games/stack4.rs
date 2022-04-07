
use serde::{Serialize, Deserialize};
use crate::games::{Player, GameState};
use crate::games::Game;
use crate::matchmaker::PlayableGame;
use std::fmt;
use std::io::BufRead;

const BOARD_SIZE: usize = 8;
const DIRS: [[i32;2];4] = [[1,0], [0,1], [-1,0], [0, -1]];
const STARTS: [[usize;2];4] = [[0,0], [BOARD_SIZE-1, 0], [BOARD_SIZE-1, BOARD_SIZE-1], [0, BOARD_SIZE-1]];


type Action = (usize, usize);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Stack4 {
    // tile on board takes up 2 bits, 0 for empty, 1 for red, 2 for yellow. 
    // starts in bottom left corner and goes row by row.
    pub board: u128,
    pub cur_player: Player,
    pub game_state: GameState,
    pub nb_moves: u32,
}

struct LegalActions {
    board: Stack4,
    side: usize,
    c: i32,
    prev_actions: u64 // Bitboard of actions already iterated over.
}

impl Stack4 {

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

    pub fn is_full(&self) -> bool {
        self.legal_actions().count() == 0
    }

    pub fn in_board(x: i32, y: i32) -> bool {
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

    // Returns board rotated by 90*n degrees
    fn rotation(&self, n: u32) -> Self {
        let n = n%4;
        let mut new_board = Stack4::new();
        for x in 0..BOARD_SIZE as i32 {
            for y in 0..BOARD_SIZE as i32 {
                let (nx, ny) = Self::rotate(x, y, n);
                new_board.set(nx as usize, ny as usize, self.get(x as usize, y as usize))
            }
        }
        Stack4 {
            board: new_board.board,
            cur_player: self.cur_player,
            game_state: self.game_state,
            nb_moves: self.nb_moves,
        }
    }

    // rotates a point 90*n degrees around the center of the board.
    // center is located at (3.5, 3.5)
    fn rotate(x: i32, y: i32, n: u32) -> (i32, i32) {
        let n = n%4;
        match n {
            0 => ( x  ,  y  ),
            1 => (-y+7,  x  ),
            2 => (-x+7, -y+7),
            3 => ( y  , -x+7),
            _ => { panic!("Impossible!")}
        }
    }

    // mirrors board around the middle of the board.
    // assumes BOARD_SIZE = 8. so if BOARD_WIDTH changes then so must this function
    pub fn mirror(&self) -> Self {
        let mut col_mask: u128 = 0;
        for _ in 0..BOARD_SIZE {
            col_mask = (col_mask<<BOARD_SIZE*2)+3;
        }
        let mut new_board:u128 = 0;
        for i in 0..BOARD_SIZE/2 {
            new_board += (self.board&(col_mask<<(i*2)) ) << ((BOARD_SIZE/2 -i)*4-2);
        }
        for i in 0..BOARD_SIZE/2 {
            new_board += (self.board&(col_mask<<( BOARD_SIZE/2 +i)*2)) >> (i)*4+2;
        }
        
        Stack4 {
            board: new_board,
            cur_player: self.cur_player,
            game_state: self.game_state,
            nb_moves: self.nb_moves,
        }
    }
}

impl Game for Stack4 {
    type Action = (usize, usize); // x,y coordinates of the placed piece.

    fn new() -> Self {
        Self {
            board: 0,
            cur_player: Player::Red,
            game_state: GameState::InProgress,
            nb_moves: 0,
        }
    }

    // Assumes that 'action' is a legal action.
    fn play_action(&mut self, action: Self::Action) {
        assert_eq!(self.game_state, GameState::InProgress);
        self.set(action.0, action.1, self.cur_player as u8);
        self.nb_moves += 1;

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
        self.game_state = GameState::InProgress;
        self.cur_player = !self.cur_player;
        self.nb_moves -= 1;
    }

    fn game_state(&self) -> GameState {
        self.game_state
    }

    fn cur_player(&self) -> Player {
        self.cur_player
    }

    fn legal_actions(&self) -> Box<dyn Iterator<Item=Action>> {
        Box::new(LegalActions { board: *self, side: 0, c: 0, prev_actions: 0})
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

    fn symmetries(&self) -> Vec<Self> {
        let mut symmetries = Vec::with_capacity(8);
        for n in 0..4 {
            let rotated = self.rotation(n);
            symmetries.push(rotated);
            symmetries.push(rotated.mirror());
        }
        symmetries
    }

    fn uid(&self) -> u128 {
        self.board
    }

    fn length(&self) -> u32 {
        self.nb_moves
    }
}

impl<'a> Iterator for LegalActions {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        let move_order = [3, 4, 2, 5, 1, 6, 0, 7];
        while self.c < BOARD_SIZE as i32 {
            let i = move_order[self.c as usize];

            while self.side < 4 {
                let dir = DIRS[self.side];
                let start = STARTS[self.side];
                let inward_direction = [-dir[1], dir[0]];
                self.side += 1;
                let cur_start = [
                    start[0] as i32+dir[0] as i32*i,
                    start[1] as i32+dir[1] as i32*i
                ];

                // moves into the board until it finds a empty square.
                for k in 0..BOARD_SIZE {
                    let cur_cord = [ 
                        (cur_start[0]+k as i32*inward_direction[0]), 
                        (cur_start[1]+k as i32*inward_direction[1])
                    ];
                    if cur_cord[0] < 0 || cur_cord[1] < 0 || cur_cord[0] >= BOARD_SIZE as i32 || cur_cord[1] >= BOARD_SIZE as i32 {
                        continue;
                    }
                    let cur_cord = [cur_cord[0] as usize, cur_cord[1] as usize];
                    // 0 represents TileStates::Empty
                    if self.board.get(cur_cord[0], cur_cord[1]) == 0 {
                        let mask = 1 << (cur_cord[0] + BOARD_SIZE*cur_cord[1]);
                        let res = self.prev_actions & mask; 
                        if res == 0 {
                            self.prev_actions += mask;
                            return Some((cur_cord[0], cur_cord[1]));
                        } else {
                            break
                        }
                    }
                }
            } 

            if self.side == 4 {
                self.c += 1;
                self.side = 0;
            }
        }
        None
    }
}

impl fmt::Debug for Stack4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        let legal_actions: Vec<_> = self.legal_actions().collect();
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

impl PlayableGame for Stack4 {
    // returns (action, is_reverse)
    fn get_action_from_user(&self) -> (Action, bool) {
        let stdin = std::io::stdin();
        let legal_actions: Vec<_> = self.legal_actions().collect();

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
}
