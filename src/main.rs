extern crate fastrand;

mod connect4;
mod search;
mod evaluators;

use connect4::{Connect4, Player, BOARD_HEIGHT, BOARD_WIDTH, GameState, Action};
use evaluators::{Evaluator, SimpleEval};
use search::{minimax_action, minimax, abpruning_action, abpruning_best_action};
use std::io::{self, BufRead};


fn main() {
    user_vs_ai();
}

fn get_move_from_minimax<T: Evaluator>(board: &Connect4, evaluator: &T) -> Action {
    /*let mut _board = board.clone();
    let mut action_values: Vec<(Action, f64)> = board.valid_moves()
        .iter()
        .map(|a| {
            //(*a, minimax_action(&mut _board, *a, 5, evaluator))
            (*a, abpruning_action(&mut _board, *a, 7, evaluator))
        })
        //.map(|a| (*a, minimax_action(&mut _board, *a, 6, evaluator)))
        .collect();
    let mx = action_values.iter().map(|(_,v)|*v).fold(-1.0/0.0, f64::max);
    println!("{:?}", action_values);
    let best_avs = action_values.iter().filter(|(_,v)| *v==mx).collect::<Vec<&(Action,f64)>>();
    best_avs[fastrand::usize(0..best_avs.len())].0
    */
    abpruning_best_action(board, 8, evaluator)
}

// returns (action, is_reverse)
fn get_move_from_user(board: &Connect4) -> (Action, bool) {
    let mut stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.as_bytes()[0] == 'z' as u8 {
            return (0, true);
        } else if let Ok(a) = line.parse::<usize>() {
            if a >= 0 && a < connect4::BOARD_WIDTH {
                if !board.is_valid_move(a) {
                    println!("Column alread full");
                    continue;
                }
                return (a, false);
            } else {
                println!("Not in range 0..{}", connect4::BOARD_WIDTH);    
            }
        } else {
            println!("Invalid input: try again");
        }
    }
    panic!("Failed to get input from user");
}

fn user_vs_user() {
    let mut board = Connect4::new();
    let evaluator = SimpleEval::new(!board.cur_player);
    loop {
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        let (action, reverse) = get_move_from_user(&board);
        if reverse {
            board.reverse_last_move();
        } else {
            board.play_move(action);
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

fn user_vs_ai() {
    let mut board = Connect4::new();
    let evaluator = SimpleEval::new(!board.cur_player);
    loop {
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        let (action, reverse) = get_move_from_user(&board);
        if reverse {
            board.reverse_last_move();
            board.reverse_last_move();
            continue
        } else {
            board.play_move(action);
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
        let action = get_move_from_minimax(&board, &evaluator);
        board.play_move(action);
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