
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
    pub board: Vec<Vec<TileStates>>,
    actions: Vec<Action>,
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
            board: vec![vec![TileStates::Empty; BOARD_HEIGHT];BOARD_WIDTH], // board[x][y] where x,y=(0,0) is the bottom left corner.
            actions: Vec::new(),
            cur_player: Player::Red,
            game_state: GameState::InProgress,
        }
    }

    pub fn play_move(&mut self, action: Action) {
        assert_eq!(self.game_state, GameState::InProgress);
        self.actions.push(action);
        if !self.is_valid_move(action) {
            return
        }
        let ap = self.action_pos(action);
        self.board[ap[0]][ap[1]] = TileStates::Full(self.cur_player);
        if self.player_won(ap) {
            self.game_state = GameState::Won(self.cur_player);
        } else if self.is_full() {
            self.game_state = GameState::Draw;
        } else {
            self.game_state = GameState::InProgress;
        }
        self.cur_player = !self.cur_player;
    }

    pub fn reverse_last_move(&mut self) {
        if let Some(last_action) = self.actions.pop() {
            let ap = self.pos_from_action(last_action);
            self.board[ap[0]][ap[1]] = TileStates::Empty;
            self.game_state = GameState::InProgress;
            self.cur_player = !self.cur_player;
        }
    }

    pub fn player_won(&self, piece_pos: [usize; 2]) -> bool {
        let directions: [[i32;2];4] = [[1,0],[0,1],[-1,1], [1,1]];
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

    pub fn in_board(&self, x:i32,y:i32) -> bool {
        x >= 0 && y >= 0 && x < BOARD_WIDTH as i32 && y < BOARD_HEIGHT as i32 
    }

    // Returns where piece will be placed if 'action' is played.
    fn action_pos(&self, action: Action) -> [usize; 2] {
        for cur_y in 0..BOARD_HEIGHT {
            if  self.board[action][cur_y] == TileStates::Empty {
                return [action, cur_y];
            }
        }
        unimplemented!()
    }

    // Returns where piece placed from last played action 'action'
    fn pos_from_action(&self, action: Action) -> [usize; 2] {
        for cur_y in 1..BOARD_HEIGHT {
            if  self.board[action][cur_y] == TileStates::Empty {
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
        self.board[action][BOARD_HEIGHT-1] == TileStates::Empty
    }

    pub fn valid_moves(&self) -> Vec<Action> {
        (0..N_ACTIONS).filter(|a|self.is_valid_move(*a)).collect()
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

impl std::ops::Index<[usize;2]> for Connect4 {
    type Output = TileStates;
    fn index(&self, idx: [usize;2]) -> &Self::Output {
        &self.board[idx[0]][idx[1]]
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
        for mv in &moves[0..moves.len()-1] {
            board.play_move(*mv);
            println!("{:?}", board);
            assert_eq!(board.game_state, GameState::InProgress);
        }
        board.play_move(moves[moves.len()-1]);
        println!("{:?}", board);
        println!("{:?}", board.actions);
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
        println!("{:?}", board.actions);
        println!("{:?}", board.game_state);
        assert_eq!(board.game_state, GameState::Won(Player::Yellow));
        board.reverse_last_move();
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
        board.reverse_last_move();
        println!("{:?}\n{:?}",old_board,board);
        assert_eq!(old_board.board, board.board);
    }
}