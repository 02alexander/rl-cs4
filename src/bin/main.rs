extern crate serde;
extern crate clap;
extern crate fastrand;
extern crate serde_json;
extern crate gamesolver;

use gamesolver::games::connect4::{Connect4, Action};
use gamesolver::games::stack4::Stack4;
use gamesolver::games::{GameState, Player};
use gamesolver::evaluators::{Evaluator, Connect4Evaluators, Stack4Evaluators, simple::SimpleEval, cnn::CNNEval};
use gamesolver::agents::{Agent, MinimaxPolicyAgent, MinimaxAgent};
use std::io::{self, BufRead};
use gamesolver::matchmaker::{MatchMaker, PlayableGame, user_vs_agent};
use gamesolver::games::{connect4, Game};
use gamesolver::qlearning::{QLearning, RL};
use gamesolver::policies::{EpsilonGreedy};
use clap::{Parser, Subcommand, ArgEnum};
use serde::{Serialize};
use serde::de::DeserializeOwned;

#[derive(Parser)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[clap(arg_enum)]
    game: Games,
    #[clap(subcommand)]
    command: Commands
}

#[derive(ArgEnum, Clone)]
enum Games {
    Connect4,
    Stack4
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

impl Commands {
    fn create(ai_file: String, model_file: Option<String>) {
        if let Some(model_file) = model_file {
            let evaluator = Connect4Evaluators::CNN(CNNEval::new(model_file));
            let policy = EpsilonGreedy::new(0.1);
            let mut ai = QLearning::new(evaluator, Box::new(policy), 0.0001);
            ai.discount = 0.95;
            ai.depth = 4;
            let serialized_ai = serde_json::to_string(&ai).unwrap();
            std::fs::write(ai_file, &serialized_ai).unwrap();
        } else {
            let evaluator = Connect4Evaluators::Simple(SimpleEval::new());
            let policy = EpsilonGreedy::new(0.1);
            let mut ai = QLearning::new(evaluator, Box::new(policy), 0.0001);
            ai.discount = 0.95;
            ai.depth = 4;
            let serialized_ai = serde_json::to_string(&ai).unwrap();
            std::fs::write(ai_file, &serialized_ai).unwrap();
        }
    }
    fn self_play<G, E>(ai_file: String, iterations: u32, progress: bool, reference_ai: Option<String>) 
        where
            G: Game,
            E: Evaluator<G> + Serialize + DeserializeOwned,
    {
        let mut ai: QLearning<E> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
        let ref_ai: Option<QLearning<E>> = if let Some(ref_ai_file) = reference_ai {
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
                    gamesolver::matchmaker::play_game(&selfagent, &refagent).last().unwrap().game_state()
                } else {
                    gamesolver::matchmaker::play_game(&refagent, &selfagent).last().unwrap().game_state()
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
    fn train_against<G, E>(ai_file: String, opponent_file: String, iterations: u32, progress: bool, scores: bool) 
        where
            G: Game,
            E: Evaluator<G>+Serialize+DeserializeOwned,
    {
        let mut ai: QLearning<E> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
        let opponent:QLearning<E> = serde_json::from_str(&std::fs::read_to_string(&opponent_file).expect("valid file")).expect("json of RL");
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
    fn compare<G, E>(ai_file1: String, ai_file2: String, nb_games: u32, depth: u32)
        where
            G: Game,
            E: Evaluator<G>+Serialize+DeserializeOwned,
    {
        let ai1: QLearning<E> = serde_json::from_str(&std::fs::read_to_string(&ai_file1).expect("valid file")).expect("json of RL");
        let ai2: QLearning<E> = serde_json::from_str(&std::fs::read_to_string(&ai_file2).expect("valid file")).expect("json of RL");
        let agenta = MinimaxPolicyAgent::new(ai1.get_evaluator(), ai1.get_policy(), depth);
        let agentb = MinimaxPolicyAgent::new(ai2.get_evaluator(), ai2.get_policy(), depth);
        let mut mm = MatchMaker::new();
        mm.add_agent(&agenta);
        mm.add_agent(&agentb);
        mm.play_n_games(nb_games);
        println!("{:?}", mm.scores());
    }
}

fn run_command<G, E>(command: Commands) 
    where
        G: PlayableGame,
        E: Evaluator<G>+Serialize+DeserializeOwned
{
    match command {
        Commands::Create{ai_file, model_file} => {
            Commands::create(ai_file, model_file);
        },
        Commands::SelfPlay {ai_file, iterations, progress, reference_ai} => {
            Commands::self_play::<G, E>(ai_file, iterations, progress, reference_ai);
        }
        Commands::TrainAgainst { ai_file, opponent_file, iterations, progress, scores} => {
            Commands::train_against::<G, E>(ai_file, opponent_file, iterations, progress, scores);
        }
        Commands::Play {ai_file} => {
            let ai: QLearning<E> = serde_json::from_str(&std::fs::read_to_string(&ai_file).expect("valid file")).expect("json of RL");
            let agenta = MinimaxPolicyAgent::new(ai.get_evaluator(), ai.get_policy(), 4);
            user_vs_agent(&agenta);
        }
        Commands::Compare {ai_file1, ai_file2, nb_games, depth} => {
            Commands::compare::<G, E>(ai_file1, ai_file2, nb_games, depth);
        }   
    }
}

fn main() { 

    /*let evaluator = SimpleEval::new();
    let agent1 = MinimaxAgent::new(&evaluator, 4);
    //let agent2 = crate::gamesolver::agents::MinimaxAgent::new(&evaluator, 4);
    crate::gamesolver::matchmaker::user_vs_user::<Stack4>();
    //user_vs_agent::<Stack4, MinimaxAgent<SimpleEval>>(&agent1);

    return ();
    */

    let args = Cli::parse();
    match args.game {
        Games::Connect4 => {
            run_command::<Connect4, Connect4Evaluators>(args.command);
        },
        Games::Stack4 => {
            run_command::<Stack4, Stack4Evaluators>(args.command);
        }
    }
}

fn _mse_cnneval<E: Evaluator<Connect4>>(evaluator: &E) -> f64 {
    // good for yellow
    let actions = vec![4, 2, 3, 5, 5, 3, 5, 5, 6, 5, 6, 2, 6, 6, 6, 3, 6, 4];
    let mut board = Connect4::new();
    for action in actions {
        board.play_action(action);
    }
    let vyellow = evaluator.value(&board, Player::Yellow);
    let vred = evaluator.value(&board, Player::Red);
    println!("vyellow={:.5}", vyellow);
    println!("vred=   {:.5}\n", vred);

    ((vyellow-1.0)*(vyellow-1.0)+(vred+1.0)*(vred+1.0))/2.0
}
