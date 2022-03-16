//#![feature(test)]
//extern crate test;


extern crate serde;
extern crate clap;
extern crate fastrand;
extern crate serde_json;
extern crate gamesolver;

use gamesolver::connect4::{Connect4, Player, BOARD_HEIGHT, BOARD_WIDTH, GameState, Action};
use gamesolver::evaluators::{Evaluator, SimpleEval, LinesEval, ConsequtiveEval, CNNEval};
use gamesolver::search::*;
use gamesolver::agents::{BatchMinimaxAgent, MinimaxAgent, MinimaxPolicyAgent};
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
    Create {
        ai_file: String,
        model_file: Option<String>
    },
    Train { 
        ai_file: String,
        iterations: u32
    },
    Play {
        ai_file: String
    },
    Compare {
        ai_file1: String,
        ai_file2: String,
        #[clap(default_value_t=100)]
        nb_games: u32,
        #[clap(default_value_t=4)]
        depth: u32,
    }
}

fn main() {

    let args = Cli::parse();

    match args.command {
        Commands::Create{ai_file, model_file} => {
            if let Some(model_file) = model_file {
                let evaluator = CNNEval::new(model_file);
                let policy = EpsilonGreedy::new(0.1);
                let mut ai = QLearning::new(Box::new(evaluator), Box::new(policy), 0.0001);
                ai.discount = 0.95;
                ai.depth = 4;
                let ai: Box<dyn RL> = Box::new(ai);
                let serialized_ai = serde_json::to_string(&ai).unwrap();
                std::fs::write(ai_file, &serialized_ai).unwrap();
            } else {
                let evaluator = SimpleEval::new();
                let policy = EpsilonGreedy::new(0.1);
                let mut ai = QLearning::new(Box::new(evaluator), Box::new(policy), 0.0001);
                ai.discount = 0.95;
                ai.depth = 4;
                let ai: Box<dyn RL> = Box::new(ai);
                let serialized_ai = serde_json::to_string(&ai).unwrap();
                std::fs::write(ai_file, &serialized_ai).unwrap();
            }
        },
        Commands::Train {ai_file, iterations} => {
            let mut ai: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
            for i in 0..iterations {
                println!("iteration: {}", i);
                ai.self_play();
            }
            let serialized_ai = serde_json::to_string(&ai).unwrap();
            std::fs::write(ai_file, &serialized_ai).unwrap();
        }
        Commands::Play {ai_file} => {
            let ai: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
            let agenta = MinimaxPolicyAgent::new(ai.get_evaluator(), ai.get_policy(), 5);
            user_vs_agent(&agenta);
        }
        Commands::Compare {ai_file1, ai_file2, nb_games, depth} => {
            let ai1: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&ai_file1).expect("valid file")).expect("json of RL");
            let ai2: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&ai_file2).expect("valid file")).expect("json of RL");
            let agenta = MinimaxPolicyAgent::new(ai1.get_evaluator(), ai1.get_policy(), depth);
            let agentb = MinimaxPolicyAgent::new(ai2.get_evaluator(), ai2.get_policy(), depth);
            let mut mm = MatchMaker::new();
            mm.add_agent(&agenta);
            mm.add_agent(&agentb);
            mm.play_n_games(nb_games);
            println!("{:?}", mm.scores());
        }   

    }
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
    //println!("vyellow={}", vyellow);
    //println!("vred={}", vred);

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
    //abpruning_best_action(board, 8, evaluator, player)
    batch_abnegamax_best_action(board, 5, 5, evaluator, player)
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

fn user_vs_agent(agent: &dyn Agent) {
    let mut board = Connect4::new();
    let p = board.cur_player;

    let mut actions = Vec::new();

    loop {
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        println!("{:?}", actions);
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