//#![feature(test)]
//extern crate test;


extern crate serde;
extern crate clap;
extern crate fastrand;
extern crate serde_json;
extern crate gamesolver;

use gamesolver::connect4::{Connect4, Player, BOARD_HEIGHT, BOARD_WIDTH, GameState, Action};
use gamesolver::evaluators::{Evaluator, SimpleEval, LinesEval, ConsequtiveEval, CNNEval};
use gamesolver::search::{minimax_action, minimax, MinimaxAgent, abpruning_action, abpruning_best_action};
use std::io::{self, BufRead};
use gamesolver::matchmaker::{Agent, MatchMaker};
use gamesolver::connect4;
use gamesolver::qlearning::{QLearning, RL};
use gamesolver::policies::{EpsilonGreedy, Greedy};
use serde::{Serialize, Deserialize};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Create {
        file: String,
    },
    Train { 
        iterations: u32
    },
    Play {
        ai_file: String
    },
    Compare {
        ai_file1: String,
        ai_file2: String,
    }
}

fn main() {
    //let cli = Cli::parse();

    let fname_ai = "cnn_ai.json";
    
    /*let evaluator = CNNEval::new(String::from("models/model.pt"));
    //let evaluator = ConsequtiveEval::new();
    let policy = EpsilonGreedy::new(0.1);
    let mut ai = QLearning::new(Box::new(evaluator), Box::new(policy), 0.001);
    ai.discount = 0.95;
    ai.depth = 4;
    let mut ai: Box<dyn RL> = Box::new(ai);
    */
    

    let serialized_ai = std::fs::read_to_string(fname_ai).unwrap();
    let mut ai: Box<dyn RL> = serde_json::from_str(&serialized_ai).unwrap();

    for i in 0..100 {
        //let lineeval = LinesEval::new();
        //let opponent = MinimaxAgent::new(&lineeval, 4);
        //ai.play_against(&opponent, Player::Red);
        ai.self_play();
        println!("{}",i);
        /*if i%200 == 0 {
            let lineval = SimpleEval::new();
            let agenta = MinimaxAgent::new(ai.get_evaluator(), 4);
            let refagent = MinimaxAgent::new(&lineval, 4);
            let mut mm = MatchMaker::new();
            mm.add_agent(&agenta);
            mm.add_agent(&refagent);
            mm.play_n_games(100);
            println!("{:?}", mm.scores());
            test_evaluator(ai.get_evaluator());
        }*/
        println!("{:?}", mse_cnneval(ai.get_evaluator()));
    }

    //let serialized_ai = serde_json::to_string(&ai).unwrap();
    //std::fs::write(fname_ai, &serialized_ai).unwrap();
    
    //user_vs_ai();
    //test_cnneval();
}

fn mse_cnneval(evaluator: &dyn Evaluator) -> f64 {
    // good for yellow
    let actions = vec![4, 2, 3, 5, 5, 3, 5, 5, 6, 5, 6, 2, 6, 6, 6, 3, 6, 4];
    let mut board = Connect4::new();
    for action in actions {
        board.play_move(action);
    }
    let vyellow = evaluator.value(&board, Player::Yellow);
    let vred = evaluator.value(&board, Player::Red);

    ((vyellow-1.0)*(vyellow-1.0)+(vred+1.0)*(vred+1.0))/2.0
}

fn test_cnneval() {
    /*let v: Vec<f32> = vec![1., 2., 3., 4., 5., 6., 7., 8.];
    let tensor = unsafe {
        let mut t = tch::Tensor::of_blob(
            v.as_ptr() as *const u8,
            &[1,2,2,2], 
            &[0,4,2,1],
            tch::Kind::Float,
            tch::Device::Cpu
        );
        //t
        tch::Tensor::clone_from_ptr(t.as_mut_ptr())
    };
    tensor.print();
    */

    let mut board = Connect4::new();
    let moves = vec![5, 4, 6, 4];
    for mv in moves {
        board.play_move(mv);
    }
    println!("{:?}", board);
    let v = board.vectorize(Player::Red);
    for i in 0..v.len() {
        print!("{:2.} ", v[i]);
        if i%connect4::BOARD_WIDTH==connect4::BOARD_WIDTH-1 {
            println!("");
        }
    }

    let mut evaluator = CNNEval::new(String::from("models/model.pt"));
    for i in 0..100 {
        let v = evaluator.value(&board, Player::Red);
        println!("{:?}", v);
        evaluator.update(&board, Player::Red, 1.0, 0.001);
    }
}

fn test_evaluator(eval: &dyn Evaluator) {
    let mut board = Connect4::new();
    let moves = vec![5, 4, 6, 4];
    for mv in moves {
        board.play_move(mv);
    }
    println!("{:?}", board);
    let v = board.vectorize(Player::Red);
    for i in 0..v.len() {
        print!("{:2.} ", v[i]);
        if i%connect4::BOARD_WIDTH==connect4::BOARD_WIDTH-1 {
            println!("");
        }
    }
    let v = eval.value(&board, Player::Red);
    println!("{:?}", v);

}

fn get_move_from_minimax<T: Evaluator>(board: &Connect4, evaluator: &T, player: Player) -> Action {
    abpruning_best_action(board, 8, evaluator, player)
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
    let evaluator = SimpleEval::new();
    let mut last_action = 0;
    loop {
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        let (action, reverse) = get_move_from_user(&board);
        if reverse {
            board.reverse_last_action(last_action);
        } else {
            last_action = action;
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
    let p = board.cur_player;
    let levaluator = ConsequtiveEval::new();
    //let evaluator = SimpleEval::new(!board.cur_player);
    
    let serialized_ai = std::fs::read_to_string("ai.json").unwrap();
    let mut ai: Box<dyn RL> = serde_json::from_str(&serialized_ai).unwrap();
    let agent = MinimaxAgent::new(ai.get_evaluator(), 4);

    let evaluator = SimpleEval::new();
    let mut actions = Vec::new();
    loop {
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        println!("{:?}", actions);
        println!("{:?}", ai.get_evaluator().value(&board, !p));
        let (action, reverse) = get_move_from_user(&board);
        if reverse {
            board.reverse_last_action(actions[actions.len()-1]);
            board.reverse_last_action(actions[actions.len()-2]);
            actions.remove(actions.len()-1);
            actions.remove(actions.len()-1);
            continue
        } else {
            actions.push(action);
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
        //let action = get_move_from_minimax(&board, &evaluator, !p);
        let action = agent.get_action(&board, !p);
        actions.push(action);
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