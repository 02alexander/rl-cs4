
use crate::connect4::{Connect4, Player, Action, GameState};
use crate::connect4;
use crate::matchmaker::Agent;
use crate::evaluators::Evaluator;
use std::collections::HashMap;

static mut count: u32 = 0;

// these two numbers must be coprime.
const TABLE_SIZE: usize = 7919;
const MULTIPLIER: usize = 7909;

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
            board.reverse_last_action(action);
        }
        return value
    } else {
        let mut value: f64 = 1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.min(_minimax(board, depth-1, true, evaluator, player));
            board.reverse_last_action(action);
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
    board.reverse_last_action(action);
    v
}

pub fn abpruning_action(
        board: &mut Connect4, action: Action, depth: u32, 
        evaluator: &dyn Evaluator, tt: &mut TranspositionTable<f64>, player: Player) -> f64 {
    board.play_move(action);
    let v = _abpruning(board, -1./0., 1./0., depth, false, evaluator, tt, player);
    board.reverse_last_action(action);
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
            board.reverse_last_action(action);
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
            board.reverse_last_action(action);
            beta = beta.min(min_value);
            if min_value <= alpha {
                break
            }
        }
        retv = min_value;

    }
    retv
}

pub fn batch_negamax_best_action(board: &Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> Action {
    let mut _board = board.clone();
    let mut avs = Vec::new();
    for action in _board.valid_moves() {
        _board.play_move(action);
        avs.push((action, -batch_negamax(&_board, depth-1, evaluator, !player)));
        _board.reverse_last_action(action);
    }
    //println!("{:?}", avs);
    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(Action,f64)>>();
    //println!("{:?}", avs);
    best_avs[fastrand::usize(0..best_avs.len())].0
}

pub fn batch_abnegamax_best_action(board: &Connect4, depth: u32, batch_depth: u32, evaluator: &dyn Evaluator, player: Player) -> Action {
    let mut _board = board.clone();
    let mut avs = Vec::new();
    for action in _board.valid_moves() {
        _board.play_move(action);
        avs.push((action, -abnegamax(&_board, depth-1, batch_depth, evaluator, !player)));
        _board.reverse_last_action(action);
    }
    //println!("{:?}", avs);
    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(Action,f64)>>();
    //println!("{:?}", avs);
    best_avs[fastrand::usize(0..best_avs.len())].0
}

pub fn negamax(board: &mut Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    if board.game_state != GameState::InProgress || depth == 0 {
        return evaluator.value(board, player);
    }
    let mut val: f64 = -1./0.;
    for action in board.valid_moves() {
        board.play_move(action);
        let v = -negamax(board, depth-1, evaluator, !player);
        board.reverse_last_action(action);
        val = val.max(v);
    }
    val
}

pub fn abnegamax(board: &Connect4, depth: u32, batch_depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    let mut _board = board.clone();
    _abnegamax(&mut _board, -1./0., 1./0., depth, batch_depth, evaluator, player)
}

fn _abnegamax(board: &mut Connect4, mut alpha: f64, mut beta: f64, depth: u32, batch_depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    if board.game_state != GameState::InProgress || depth == 0 {
        return evaluator.value(board, player);
    }
    let mut val: f64 = -1./0.;
    for action in board.valid_moves() {
        board.play_move(action);
        //let v = -_abnegamax(board, -beta, -alpha, depth-1, batch_depth, evaluator, !player);
        let v = if depth <= batch_depth+1 {
            -batch_negamax(board, depth-1, evaluator, !player)
        } else {
            -_abnegamax(board, -beta, -alpha, depth-1, batch_depth, evaluator, !player)
        };
        val = val.max(v);
        board.reverse_last_action(action);
        alpha = alpha.max(val);
        if alpha >= beta {
            break;
        }
    }
    val
}

pub fn batch_negamax(board: &Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player) -> f64 {
    let mut _board = board.clone();
    let leafs = leafs(&mut _board, depth);
    let mut vals: HashMap<u128, f64> = HashMap::new();
    let mut leaf_vals = Vec::with_capacity(leafs.len());
    
    // compute leaf values in batch
    if depth % 2 == 0 {
        leaf_vals.append(&mut evaluator.values(&leafs, player));
    } else {
        leaf_vals.append(&mut evaluator.values(&leafs, !player));
    }

    for (i,leaf_val) in leaf_vals.iter().enumerate() {
        vals.insert(leafs[i].board,*leaf_val);
    }

    negamax_from_hashmap(&mut _board, depth, evaluator, player, &vals)

}

fn negamax_from_hashmap(board: &mut Connect4, depth: u32, evaluator: &dyn Evaluator, player: Player, hmap: &HashMap<u128, f64>) -> f64 {
    if board.game_state != GameState::InProgress {
        return evaluator.value(board, player);
    }
    if depth == 0 {
        return hmap[&board.board];
    }
    let mut val: f64 = -1./0.;
    for action in board.valid_moves() {
        board.play_move(action);
        let v = -negamax_from_hashmap(board, depth-1, evaluator, !player, &hmap);
        val = val.max(v);
        board.reverse_last_action(action);
    }
    val
}

pub fn leafs(board: &mut Connect4, depth: u32) -> Vec<Connect4> {
    if depth == 0 {
        return vec![board.clone()];        
    }
    if board.game_state != GameState::InProgress {
        return Vec::new();
    }
    let mut ret = Vec::new();
    for action in board.valid_moves() {
        board.play_move(action);
        ret.append(&mut leafs(board, depth-1));
        board.reverse_last_action(action);
    }
    ret
}