
use crate::games::{Player, GameState, Game};
use crate::evaluators::{Evaluator};
use std::collections::HashMap;

pub static mut LEAF_COUNT: u32 = 0;

// these two numbers must be coprime.
const TABLE_SIZE: usize = 104723;
const MULTIPLIER: usize = 48619;

pub struct TranspositionTable<T> {
    pub table: Vec<Option<(u128, T)>>,
}

pub fn hash(board: u128) -> usize {
    (board * MULTIPLIER as u128 % TABLE_SIZE as u128) as usize
}


impl<T: Copy> TranspositionTable<T> {
    pub fn new() -> TranspositionTable<T> {
        TranspositionTable {
            table: vec![None; TABLE_SIZE],
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


pub fn abnegamax_best_action<T, E>(board: &T, depth: u32, evaluator: &E, player: Player) -> T::Action 
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
    let mut tt = TranspositionTable::new();
    let mut _board = board.clone();
    let mut avs = Vec::new();
    for action in _board.legal_actions() {
        _board.play_action(action);
        avs.push((action, -abnegamax(&_board, depth-1, 0, evaluator, !player, Some(&mut tt))));
        _board.reverse_last_action(action);
    }
    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(T::Action,f64)>>();
    best_avs[fastrand::usize(0..best_avs.len())].0
}


pub fn batch_negamax_best_action<T, E>(board: &T, depth: u32, evaluator: &E, player: Player) -> T::Action 
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
    let mut _board = board.clone();
    let mut avs = Vec::new();
    for action in _board.legal_actions() {
        _board.play_action(action);
        avs.push((action, -batch_negamax(&_board, depth-1, evaluator, !player)));
        _board.reverse_last_action(action);
    }
    //println!("{:?}", avs);
    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(T::Action,f64)>>();
    //println!("{:?}", avs);
    best_avs[fastrand::usize(0..best_avs.len())].0
}

pub fn batch_abnegamax_best_action<T, E>(board: &T, depth: u32, batch_depth: u32, evaluator: &E, player: Player) -> T::Action 
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
    let mut tt = TranspositionTable::new();
    let mut _board = board.clone();
    let mut avs = Vec::new();
    for action in _board.legal_actions() {
        _board.play_action(action);
        avs.push((action, -abnegamax(&_board, depth-1, batch_depth, evaluator, !player, Some(&mut tt))));
        _board.reverse_last_action(action);
    }
    //println!("{:?}", avs);
    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(T::Action,f64)>>();
    //println!("{:?}", avs);
    best_avs[fastrand::usize(0..best_avs.len())].0
}

pub fn negamax<T, E>(board: &mut T, depth: u32, evaluator: &E, player: Player) -> f64 
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
    if board.game_state() != GameState::InProgress || depth == 0 {
        return evaluator.value(board, player);
    }
    let mut val: f64 = -1./0.;
    for action in board.legal_actions() {
        board.play_action(action);
        let v = -negamax(board, depth-1, evaluator, !player);
        board.reverse_last_action(action);
        val = val.max(v);
    }
    val
}

pub fn abnegamax<T, E>(board: &T, depth: u32, batch_depth: u32, 
                       evaluator: &E, player: Player, tt: Option<&mut TranspositionTable<f64>>) -> f64 
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
    let mut _board = board.clone();
    if let Some(t) = tt {
        _abnegamax(&mut _board, -1./0., 1./0., depth, batch_depth, evaluator, player, t)
    } else {
        let mut t = TranspositionTable::new();
        _abnegamax(&mut _board, -1./0., 1./0., depth, batch_depth, evaluator, player, &mut t)
    }
}

fn _abnegamax<T,E>(board: &mut T, mut alpha: f64, mut beta: f64, depth: u32, batch_depth: u32, 
                   evaluator: &E, player: Player, tt: &mut TranspositionTable<f64>) -> f64
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
    if board.game_state() != GameState::InProgress || depth == 0 {
        unsafe {LEAF_COUNT += 1};
        return evaluator.value(board, player);
    }
    let mut max = 1./0.;
    if let Some(val) = tt.get(board.uid()) {
        max = val;
    }
    if beta > max {
        beta = max;
        if alpha >= beta {
            return beta;
        }
    }
    let mut val: f64 = -1./0.;
    for action in board.legal_actions() {
        board.play_action(action);
        let v = if depth <= batch_depth {
            -batch_negamax(board, depth-1, evaluator, !player)
        } else {
            -_abnegamax(board, -beta, -alpha, depth-1, batch_depth, evaluator, !player, tt)
        };
        val = val.max(v);
        board.reverse_last_action(action);
        alpha = alpha.max(val);
        if alpha >= beta {
            break;
        }
    }
    tt.set(board.uid(), alpha);
    alpha
}

pub fn batch_negamax<T, E>(board: &T, depth: u32, evaluator: &E, player: Player) -> f64 
    where 
        T: Game, 
        E: Evaluator<T>,
        T::Action: Copy
{
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
        vals.insert(leafs[i].uid(),*leaf_val);
    }

    negamax_from_hashmap(&mut _board, depth, evaluator, player, &vals)

}

fn negamax_from_hashmap<T, E>(board: &mut T, depth: u32, evaluator: &E, player: Player, hmap: &HashMap<u128, f64>) -> f64
where 
    T: Game, 
    E: Evaluator<T>,
    T::Action: Copy
{
    if board.game_state() != GameState::InProgress {
        return evaluator.value(board, player);
    }
    if depth == 0 {
        return hmap[&board.uid()];
    }
    let mut val: f64 = -1./0.;
    for action in board.legal_actions() {
        board.play_action(action);
        let v = -negamax_from_hashmap(board, depth-1, evaluator, !player, &hmap);
        val = val.max(v);
        board.reverse_last_action(action);
    }
    val
}

pub fn leafs<T>(board: &mut T, depth: u32) -> Vec<T> 
    where 
        T: Game,
        T::Action: Copy
{
    if depth == 0 {
        return vec![board.clone()];        
    }
    if board.game_state() != GameState::InProgress {
        return Vec::new();
    }
    let mut ret = Vec::new();
    for action in board.legal_actions() {
        board.play_action(action);
        ret.append(&mut leafs(board, depth-1));
        board.reverse_last_action(action);
    }
    ret
}

#[cfg(test)]
mod test {
    use crate::games::connect4::Connect4;
    use crate::games::Game;
    use super::*;
    #[test]
    fn transposition_table() {

        let mut tt: TranspositionTable<f64> = TranspositionTable::new();
        let mut board = Connect4::new();
        board.play_action(4);
        board.play_action(5);
        tt.set(board.uid(), 1.0);
        assert_eq!(tt.get(board.uid()), Some(1.0));
    }
}