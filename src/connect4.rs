
use std::fmt;
use std::ops;

pub const BOARD_WIDTH: usize = 7;
pub const BOARD_HEIGHT: usize = 6;
pub const N_ACTIONS: usize = BOARD_WIDTH;

pub const REWARD_LOSE: f64 = -1.0;
pub const REWARD_WIN: f64 = 1.0;
pub const REWARD_DRAW: f64 = 0.0;

pub type Action = usize; // a value in the range of [0,BOARD_WIDTH)

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileStates {
    Empty,
    Full(Player),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Player {
    Red,
    Yellow,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
    Won(Player),
    Draw,
    InProgress
}

#[derive(Clone)]
pub struct Connect4 {
    board: Vec<Vec<TileStates>>,
    last_action: Action,
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
            board: vec![vec![TileStates::Empty; BOARD_HEIGHT];BOARD_WIDTH], // board[x][y] where x,y=(0,0) is the bottom left corner.
            last_action: 0,
            game_state: GameState::InProgress,
        }
    }

    // returns false if that column is full and thus not a valid position.
    /*fn place_piece(&mut self, action: Action, player: Player) -> bool {
        assert!(action < BOARD_WIDTH);
        for cur_y in 0..BOARD_HEIGHT {
            if  self.board[action][cur_y] == TileStates::Empty {
                self.board[action][cur_y] = TileStates::Full(player);
                return true;
            }
        }
        false
    }*/

    pub fn play_move(&mut self, action: Action, player: Player) {
        assert_eq!(self.game_state, GameState::InProgress);
        self.last_action = action;
        if !self.is_valid_move(action) {
            return
        }
        let ap = self.action_pos(action);
        self.board[ap[0]][ap[1]] = TileStates::Full(player);
        if self.player_won(ap) {
            self.game_state = GameState::Won(player);
        } else if self.is_full() {
            self.game_state = GameState::Draw;
        } else {
            self.game_state = GameState::InProgress;
        }
    }

    pub fn player_won(&self, piece_pos: [usize; 2]) -> bool {
        let directions: [[i32;2];3] = [[1,0],[0,1],[-1,1]];
        let player = if let TileStates::Full(p) = self.board[piece_pos[0]][piece_pos[1]] {
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
                } else if TileStates::Full(player) != self.board[curx as usize][cury as usize] {
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
                } else if TileStates::Full(player) != self.board[curx as usize][cury as usize] {
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



    fn in_board(&self, x:i32,y:i32) -> bool {
        x >= 0 && y >= 0 && x < BOARD_WIDTH as i32 && y < BOARD_HEIGHT as i32 
    }

    fn action_pos(&self, action: Action) -> [usize; 2] {
        for cur_y in 0..BOARD_HEIGHT {
            if  self.board[action][cur_y] == TileStates::Empty {
                return [action, cur_y];
            }
        }
        [0,0]
    }

    /*pub fn player_won(&self, player: Player) -> bool {
        let directions = vec![vec![1,0], vec![0,1], vec![1,1], vec![-1,1]];
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                //let start = Vector::new(vec![x as i32 ,y as i32]);
                let start = vec![x as i32 ,y as i32];
                for direction in &directions {
                    for i in 0..4 {
                        let offset = vec![direction[0]*i,direction[1]*i];
                        //let p = &start + &offset;
                        let p = vec![start[0]+offset[0], start[1]+offset[1]];
                        if p[0] < 0 || p[0] >= BOARD_WIDTH as i32 || p[1] < 0 || p[1] >= BOARD_HEIGHT as i32 {
                            break;
                        }
                        if self.board[p[0] as usize][p[1] as usize] != TileStates::Full(player) {
                            break;
                        }
                        if i == 3 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }*/

    pub fn is_full(&self) -> bool {
        !(0..N_ACTIONS).any(|action|self.is_valid_move(action))
    }

    pub fn is_valid_move(&self, action: Action) -> bool {
        assert!(action < BOARD_WIDTH);
        self.board[action][BOARD_HEIGHT-1] == TileStates::Empty
    }

    pub fn next_board(&self, action: Action, player: Player) -> Connect4 {
        let mut board = self.clone();
        board.place_piece(action, player);
        board
    }

    pub fn vectorize(&self, player: Player) -> Vec<f64> {
        let mut v = Vec::with_capacity(BOARD_WIDTH*BOARD_HEIGHT);
        
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if self.board[x][y] == TileStates::Full(player) {
                    v.push(1.0);
                } else if self.board[x][y] == TileStates::Full(!player) {
                    v.push(-1.0);
                } else {
                    v.push(0.0);
                }
            }
        }
        v
    }
}


impl fmt::Debug for Connect4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for y in (0..BOARD_HEIGHT).rev() {
            for x in 0..BOARD_WIDTH {
                match self.board[x][y] {
                    TileStates::Empty => s.push_str("# "),
                    TileStates::Full(Player::Red) => {
                        s.push_str("\x1b[30;41m \x1b[0m ");
                    }
                    TileStates::Full(Player::Yellow) => {
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
        let mut cur_player = Player::Red;
        for mv in &moves[0..moves.len()-1] {
            board.play_move(*mv, cur_player);
            cur_player = !cur_player;
            assert_eq!(board.game_state, GameState::InProgress);
        }
        board.play_move(moves[moves.len()-1], cur_player);
        println!("{:?}", board);
        println!("{:?}", board.last_action);
        println!("{:?}", cur_player);
        println!("{:?}", board.game_state);
        assert_eq!(board.game_state, GameState::Won(cur_player));
    }
}