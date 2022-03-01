//#![feature(test)]
//extern crate test;


extern crate serde;
extern crate clap;
extern crate fastrand;
extern crate serde_json;
extern crate gamesolver;

use gamesolver::connect4::{Connect4, Player, BOARD_HEIGHT, BOARD_WIDTH, GameState, Action};
use gamesolver::evaluators::{Evaluator, SimpleEval, LinesEval, ConsequtiveEval};
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

    /*let evaluator = ConsequtiveEval::new();
    let policy = EpsilonGreedy::new(0.1);
    let mut ai = QLearning::new(Box::new(evaluator), Box::new(policy), 0.0001);
    ai.discount = 0.95;
    ai.depth = 4;
    let mut ai: Box<dyn RL> = Box::new(ai);
    */
    

    let serialized_ai = std::fs::read_to_string("ai.json").unwrap();
    let mut ai: Box<dyn RL> = serde_json::from_str(&serialized_ai).unwrap();

    for i in 0..2001 {
        //let lineeval = LinesEval::new();
        //let opponent = MinimaxAgent::new(&lineeval, 4);
        //ai.play_against(&opponent, Player::Red);
        ai.self_play();

        if i%200 == 0 {
            let lineval = LinesEval::new();
            let agenta = MinimaxAgent::new(ai.get_evaluator(), 4);
            let refagent = MinimaxAgent::new(&lineval, 4);
            let mut mm = MatchMaker::new();
            mm.add_agent(&agenta);
            mm.add_agent(&refagent);
            mm.play_n_games(100);
            println!("{:?}", mm.scores());
            println!("{:?}", ai.get_evaluator().get_params());
        }
        //ai.self_play();
    }

    let serialized_ai = serde_json::to_string(&ai).unwrap();
    std::fs::write("ai.json", &serialized_ai).unwrap();

    // let serialized_ai = std::fs::read_to_string("ai.json").unwrap();
    // let deserialized_ai: Box<dyn RL> = serde_json::from_str(&serialized_ai).unwrap();
    

    //user_vs_ai();
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
    let p = board.cur_player;
    let levaluator = ConsequtiveEval::new();
    //let evaluator = SimpleEval::new(!board.cur_player);
    let evaluator = SimpleEval::new();
    loop {
        println!("{:?}", board.actions);
        println!("{:?}", board);
        println!("{:?}", board.game_state);
        println!("{:?}", levaluator.value(&board, !p));
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
        let action = get_move_from_minimax(&board, &evaluator, !p);
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