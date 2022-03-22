extern crate serde;
extern crate clap;
extern crate fastrand;
extern crate serde_json;
extern crate gamesolver;

use gamesolver::connect4::{Connect4, Player, GameState, Action};
use gamesolver::evaluators::{Evaluator, SimpleEval, CNNEval};
use gamesolver::agents::{MinimaxPolicyAgent};
use std::io::{self, BufRead};
use gamesolver::matchmaker::{Agent, MatchMaker};
use gamesolver::connect4;
use gamesolver::qlearning::{QLearning, RL};
use gamesolver::policies::{EpsilonGreedy};
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

        /// File containing libtorch model if you want to create for example a CNN evaluator. 
        model_file: Option<String>
    },
    SelfPlay { 
        /// AI that is to be trained.
        ai_file: String,
        #[clap(short, long, default_value_t=20)]
        iterations: u32,

        #[clap(short, long)]
        /// Print which iteration it's on.
        progress: bool,
        
        #[clap(short, long)]
        reference_ai: Option<String>,
    },
    TrainAgainst {
        /// AI that is to be trained.
        ai_file: String,

        opponent_file: String,

        #[clap(short, long, default_value_t=20)]
        iterations: u32,

        #[clap(short, long)]
        /// Print which iteration it's on.
        progress: bool,

        #[clap(short, long)]
        /// Print the score of every game at the end.
        scores: bool,
    },
    /// Lets user play a game against the AI.
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

fn _test_update() {
    let mut eval = CNNEval::new("models/model.pt".to_owned());
    let mut board = Connect4::new();
    for action in vec![1,2,1,4,2,5,3] {
        board.play_move(action);
    }
    for _ in 0..10000 {
        let grad = eval.gradient(&board, !Player::Red);
        let yhat = eval.value(&board, !Player::Red);
        let y = 1.0;
        let deltas: Vec<_> = grad.iter().map(|g| g*(y-yhat)*0.001).collect();
        eval.apply_update(&deltas);
        println!("{:.5}", (y-yhat).powf(2.0));
    }
    
}

fn main() {

    //_test_update();

    

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
        Commands::SelfPlay {ai_file, iterations, progress, reference_ai} => {
            let mut ai: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
            let ref_ai: Option<Box<dyn RL>> = if let Some(ref_ai_file) = reference_ai {
                Some(serde_json::from_str(&std::fs::read_to_string(&ref_ai_file).expect("valid file")).expect("json of RL"))
            } else {
                None
            };
            let mut scores: Vec<f64> = Vec::new();
            for i in 0..iterations {
                if progress {
                    println!("iteration: {}", i);
                }
                ai.self_play();
                if let Some (ref ref_ai) = ref_ai {
                    let selfagent = MinimaxPolicyAgent::new(ai.get_evaluator(), ai.get_policy(), 3);
                    let refagent = MinimaxPolicyAgent::new(ref_ai.get_evaluator(), ref_ai.get_policy(), 3);
                    let b = fastrand::bool();
                    let result = if b {
                        gamesolver::matchmaker::play_game(&selfagent, &refagent).last().unwrap().game_state
                    } else {
                        gamesolver::matchmaker::play_game(&refagent, &selfagent).last().unwrap().game_state
                    };
                    let score = match result {
                        GameState::Won(Player::Red) => 1.0,
                        GameState::Won(Player::Yellow) => -1.0,
                        GameState::Draw => 0.0,
                        GameState::InProgress => panic!("Game ended while still in progress.")
                    };
                    let score = if b {score} else {-score};
                    scores.push(score);
                }
            }
            if scores.len() != 0 {
                println!("{:?}", scores);
            }
            let serialized_ai = serde_json::to_string(&ai).unwrap();
            std::fs::write(ai_file, &serialized_ai).unwrap();
        }
        Commands::TrainAgainst { ai_file, opponent_file, iterations, progress, scores} => {
            let mut ai: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
            let opponent: Box<dyn RL> = serde_json::from_str(&std::fs::read_to_string(&opponent_file).expect("valid file")).expect("json of RL");
            for i in 0..iterations {
                if progress {
                    println!("iteration: {}", i);
                }
                let opponent = MinimaxPolicyAgent::new(opponent.get_evaluator(), opponent.get_policy(), opponent.get_depth());
                ai.play_against(&opponent);
            }
            if scores {
                println!("{:?}", ai.scores().unwrap());
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

fn _mse_cnneval(evaluator: &dyn Evaluator) -> f64 {
    // good for yellow
    let actions = vec![4, 2, 3, 5, 5, 3, 5, 5, 6, 5, 6, 2, 6, 6, 6, 3, 6, 4];
    let mut board = Connect4::new();
    for action in actions {
        board.play_move(action);
    }
    let vyellow = evaluator.value(&board, Player::Yellow);
    let vred = evaluator.value(&board, Player::Red);
    println!("vyellow={:.5}", vyellow);
    println!("vred=   {:.5}\n", vred);

    ((vyellow-1.0)*(vyellow-1.0)+(vred+1.0)*(vred+1.0))/2.0
}

// returns (action, is_reverse)
fn get_move_from_user(board: &Connect4) -> (Action, bool) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.as_bytes()[0] == 'z' as u8 {
            return (0, true);
        } else if let Ok(a) = line.parse::<usize>() {
            if a < connect4::BOARD_WIDTH {
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

fn _user_vs_user() {
    let mut board = Connect4::new();
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