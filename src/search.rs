
use crate::connect4::{Connect4, Player, Action, GameState};
use crate::connect4;
use crate::evaluators::Evaluator;

static mut count: u32 = 0;

pub fn minimax<T: Evaluator>(board: &Connect4, depth: u32, evaluator: &T) -> f64 {
    let mut _board = board.clone();
    unsafe { 
        count = 0;
    }
    let v = _minimax(&mut _board, depth, true, evaluator);
    unsafe { 
        println!("minimax  count={:?}", count); 
    }
    v
}

fn _minimax<T: Evaluator>(board: &mut Connect4, depth: u32, ismaximizing: bool, evaluator: &T) -> f64 {
    //let old_board = board.clone();
    if depth == 0 || board.game_state != GameState::InProgress {
        let v = evaluator.value(&board); 
        unsafe { 
            count += 1;
        }
        return v;
    } else if ismaximizing {
        let mut value: f64 = -1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.max(_minimax(board, depth-1, false, evaluator));
            board.reverse_last_move();
        }
        /*if old_board.board != board.board {
            println!("{:?}{:?} depth={}", old_board, board, depth);
            panic!("oh no");
        }*/
        return value
    } else {
        let mut value: f64 = 1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.min(_minimax(board, depth-1, true, evaluator));
            board.reverse_last_move();
        }
        /*if old_board.board != board.board {
            println!("{:?}{:?} depth={}", old_board, board, depth);
            panic!("oh no");
        }*/
        return value
    }
}

pub fn minimax_action<T: Evaluator>(board: &mut Connect4, action: Action, depth: u32, evaluator: &T) -> f64 {
    board.play_move(action);
    unsafe { 
        count = 0;
    }
    let v = _minimax(board, depth, false, evaluator);
    unsafe { 
        println!("minimax count={:?}", count); 
    }
    board.reverse_last_move();
    v
}

pub fn abpruning_action<T: Evaluator>(
        board: &mut Connect4, action: Action, depth: u32, evaluator: &T) -> f64 {
    board.play_move(action);
    let v = _abpruning(board, -1./0., 1./0., depth, false, evaluator);
    board.reverse_last_move();
    v
}

pub fn abpruning_best_action<T: Evaluator>(
        board: &Connect4, depth: u32, evaluator: &T) -> Action {

    
    unsafe { 
        count = 0;
    }
    let mut _board = board.clone();

    let mut avs = Vec::new();
    for action in board.valid_moves() {
        let v = abpruning_action(&mut _board, action, depth-1, evaluator);
        avs.push((action,v));
    }

    unsafe { 
        //println!("count={}", count);
    }

    let mx = avs.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    let best_avs = avs.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(Action,f64)>>();
    //println!("{:?}", best_avs);
    best_avs[fastrand::usize(0..best_avs.len())].0

}

fn _abpruning<T: Evaluator>(board: &mut Connect4, mut alpha: f64, mut beta: f64, depth: u32, ismaximizing: bool, evaluator: &T) -> f64 {
    if depth == 0 || board.game_state != GameState::InProgress {
        unsafe { 
            count += 1;
        }
        return evaluator.value(&board)
    } else if ismaximizing {
        let mut max_value: f64 = -1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            max_value = max_value.max(_abpruning(board, alpha, beta, depth-1, false, evaluator));
            board.reverse_last_move();
            alpha = alpha.max(max_value);
            
            if max_value >= beta {
                break
            }
        }
        return max_value
    } else {
        let mut min_value: f64 = 1.0/0.0;
        for action in board.valid_moves() {
            board.play_move(action);
            min_value = min_value.min(_abpruning(board, alpha, beta, depth-1, true, evaluator));
            board.reverse_last_move();
            beta = beta.min(min_value);
            if min_value <= alpha {
                break
            }
        }
        return min_value
    }
}