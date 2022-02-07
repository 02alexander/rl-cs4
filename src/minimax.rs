
use crate::connect4::{Connect4, Player, Action, GameState};
use crate::connect4;
use crate::evaluators::Evaluator;


pub fn minimax<T: Evaluator>(board: &Connect4, depth: u32, evaluator: &T) -> f64 {
    let mut _board = board.clone();
    _minimax(&mut _board, depth, true, evaluator)    
}

pub fn minimax_action<T: Evaluator>(board: &mut Connect4, action: Action, depth: u32, evaluator: &T) -> f64 {
    board.play_move(action);
    let v = _minimax(board, depth, false, evaluator);
    board.reverse_last_move();
    v
}

fn _minimax<T: Evaluator>(board: &mut Connect4, depth: u32, ismaximizing: bool, evaluator: &T) -> f64 {
    let old_board = board.clone();
    if depth == 0 || board.game_state != GameState::InProgress {
        let v = evaluator.value(&board); 
        //println!("{:?} v={} {}\n", board,v, !ismaximizing);
        /*if v != 0.0 {
            println!("{:?} v={} {}\n", board,v, !ismaximizing);
        }*/
        return v;
    } else if ismaximizing {
        let mut value: f64 = -10000.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.max(_minimax(board, depth-1, false, evaluator));
            board.reverse_last_move();
        }
        if old_board.board != board.board {
            println!("{:?}{:?} depth={}", old_board, board, depth);
            panic!("oh no");
        }
        return value
    } else {
        let mut value: f64 = 10000.0;
        for action in board.valid_moves() {
            board.play_move(action);
            value = value.min(_minimax(board, depth-1, true, evaluator));
            board.reverse_last_move();
        }
        if old_board.board != board.board {
            println!("{:?}{:?} depth={}", old_board, board, depth);
            panic!("oh no");
        }
        return value
    }
}