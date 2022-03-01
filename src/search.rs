
use crate::connect4::{Connect4, Player, Action, GameState};
use crate::connect4;
use crate::matchmaker::Agent;
use crate::evaluators::Evaluator;

static mut count: u32 = 0;

// these two numbers must be coprime.
const TABLE_SIZE: usize = 7919;
const MULTIPLIER: usize = 7909;


pub struct MinimaxAgent<'a> {
    evaluator: &'a dyn Evaluator,
    depth: u32,
}

impl<'a> MinimaxAgent<'a> {
    pub fn new(evaluator: &'a dyn Evaluator, depth: u32) -> Self {
        MinimaxAgent {
            evaluator,
            depth,
        }
    }
}

impl<'a> Agent for MinimaxAgent<'a> {
    fn get_action(&self, board: &Connect4, player: Player) -> Action {
        abpruning_best_action(board, self.depth, &*self.evaluator, player)
    }
}

pub struct TranspositionTable<T> {
    table: [Option<(u128, T)>;TABLE_SIZE],
}

pub fn hash(board: u128) -> usize {
    (board * MULTIPLIER as u128 % TABLE_SIZE as u128) as usize
}


impl<T: Copy> TranspositionTable<T> {
    pub fn new() -> TranspositionTable<T> {
        TranspositionTable {
            table: [None; TABLE_SIZE],
        }
    }

    pub fn get(&self, board: u128) -> Option<T> {
        if let Some((b, val)) = &self.table[hash(board)] {
            if board == *b {
                Some(*val)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set(&mut self, board: u128, value: T) {
        self.table[hash(board)] = Some((board, value));
    }
}

pub fn minimax(board: &Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    let mut _board = board.clone();
    unsafe { 
        count = 0;
    }
    let v = _minimax(&mut _board, depth, true, evaluator, player);
    unsafe { 
        //println!("minimax  count={:?}", count); 
    }
    v
}

fn _minimax(board: &mut Connect4, depth: u32, ismaximizing: bool, evaluator: &dyn Evaluator, player: Player) -> f64 {
    //let old_board = board.clone();
    if depth == 0 || board.game_state != GameState::InProgress {
        let v = evaluator.value(&board, player); 
        unsafe { 
            count += 1;
        }
        return v;
    } else if ismaximizing {
        let mut value: f64 = -1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.max(_minimax(board, depth-1, false, evaluator, player));
            board.reverse_last_move();
        }
        return value
    } else {
        let mut value: f64 = 1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.min(_minimax(board, depth-1, true, evaluator, player));
            board.reverse_last_move();
        }
        return value
    }
}

pub fn minimax_action(board: &mut Connect4, action: Action, depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    board.play_move(action);
    unsafe { 
        count = 0;
    }
    let v = _minimax(board, depth, false, evaluator, player);
    unsafe { 
        println!("minimax count={:?}", count); 
    }
    board.reverse_last_move();
    v
}

pub fn abpruning_action(
        board: &mut Connect4, action: Action, depth: u32, 
        evaluator: &dyn Evaluator, tt: &mut TranspositionTable<f64>, player: Player) -> f64 {
    board.play_move(action);
    let v = _abpruning(board, -1./0., 1./0., depth, false, evaluator, tt, player);
    board.reverse_last_move();
    v
}

pub fn abpruning_action_values(
        board: &mut Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> Vec<(Action, f64)> {
    let mut avs = Vec::with_capacity(connect4::N_ACTIONS);
    let mut tt = TranspositionTable::new();
    for action in board.valid_moves() {
        let v = abpruning_action(board, action, depth, evaluator, &mut tt, player);
        avs.push((action,v));
    }
    avs
}

pub fn abpruning_best_action(
        board: &Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> Action {

    let mut _board = board.clone();
    let mut avs = abpruning_action_values(&mut _board, depth-1, evaluator, player);

    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(Action,f64)>>();
    //println!("{:?}", avs);
    best_avs[fastrand::usize(0..best_avs.len())].0
}

pub fn abpruning_value(
    board: &Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    let mut tt = TranspositionTable::new();
    _abpruning(&mut board.clone(), -1./0., 1./0., depth, true, evaluator, &mut tt, player)
}


fn _abpruning(board: &mut Connect4, mut alpha: f64, mut beta: f64, 
    depth: u32, ismaximizing: bool, evaluator: &dyn Evaluator, 
    tt: &mut TranspositionTable<f64>, player: Player) -> f64 {
    let mut retv = 0.0;
    if depth == 0 || board.game_state != GameState::InProgress {
        unsafe { 
            count += 1;
        }
        return evaluator.value(&board, player);
    }
    let cached = tt.get(board.board);
    if let Some(val) = cached {
        //println!("used {}, {} {}", val, beta, hash(board.board));
        if val > beta {
            //println!("kjfg");
            return val
        }
    } 
    if ismaximizing {
        let mut max_value: f64 = -1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            max_value = max_value.max(_abpruning(board, alpha, beta, depth-1, false, evaluator, tt, player));
            board.reverse_last_move();
            alpha = alpha.max(max_value);
            
            if max_value >= beta {
                break
            }
        }
        retv = max_value;
        if let Some(v) = cached {
            if max_value > v {
                tt.set(board.board, max_value);
            }
        } else {
            tt.set(board.board, max_value);
        }
    } else {
        let mut min_value: f64 = 1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            min_value = min_value.min(_abpruning(board, alpha, beta, depth-1, true, evaluator, tt, player));
            board.reverse_last_move();
            beta = beta.min(min_value);
            if min_value <= alpha {
                break
            }
        }
        retv = min_value;

    }

    /*if let Some(val) = cached {
        if val < retv {
            //println!("replace");
            tt.set(board.board, retv);
        }
    } else {
        //println!("inserted {} at board {} ", retv, hash(board.board));
        tt.set(board.board, retv);
    }*/

    retv
}