
use crate::evaluators::{Evaluator, ConsequtiveEval, LinesEval};
use crate::connect4::{Connect4, Player, N_ACTIONS, Action, GameState};
use crate::policies::Policy;
use crate::matchmaker::{Agent, play_game};
use crate::search::{abpruning_value, batch_negamax};
use crate::agents::{BatchMinimaxAgent, MinimaxAgent, MinimaxPolicyAgent};
use serde::{Serialize, Deserialize};

#[typetag::serde(tag = "type")]
pub trait RL {
    fn update(&mut self, game_hist: &Vec<Connect4>, player: Player);
    fn self_play(&mut self);
    fn play_against(&mut self, opponent: &dyn Agent);
    fn get_evaluator<'a>(&'a self) -> &'a dyn Evaluator;
}


#[derive(Serialize, Deserialize)]
pub struct QLearning {
    evaluator: Box<dyn Evaluator>,
    exploration_policy: Box<dyn Policy>, // exploration policy
    pub step_size: f64,
    pub discount: f64,
    pub depth: u32, // depth to search during training.
}

impl QLearning { 
    pub fn new(evaluator: Box<dyn Evaluator>, exploration_policy: Box<dyn Policy>, step_size: f64) -> QLearning {
        QLearning {
            evaluator,
            exploration_policy,
            step_size,
            discount: 1.0,
            depth: 4,
        }
    }
}

#[typetag::serde]
impl RL for QLearning {
    
    // Applies updates from game_hist where self is 'player'
    fn update(&mut self, game_hist: &Vec<Connect4>, player: Player) {

        let mut states = Vec::with_capacity(game_hist.len()/2+1);
        for board in game_hist {
            if board.cur_player == player {
                states.push(board);
            }
        }
        for i in 0..(states.len()-1) {
            //let grad = self.evaluator.gradient(&states[i], player);
            
            let next_state = &states[i+1];
            let actions = next_state.valid_moves();
            let target_av = if i == states.len()-2 {
                match game_hist.last().unwrap().game_state {
                    GameState::Won(p) => {
                        if p == player {
                            1.0
                        } else {
                            -1.0
                        }
                    },
                    GameState::Draw => {
                        0.0
                    },
                    GameState::InProgress => {
                        panic!("last state is in progress")
                    }
                }
                //self.evaluator.value(&next_state, player)
            } else {

                //let v = abpruning_value(&next_state, self.depth, &*self.evaluator, player);
                let v = batch_negamax(&next_state, self.depth, &*self.evaluator, player);
                if v == 1./0. {
                    1.0
                } else if v == -1./0. {
                    -1.0
                } else {
                    v
                }
            };
            
            //let current_av = self.evaluator.value(&states[i], player);
            //let deltas = grad.iter().map(|g| g*(self.discount*target_av-current_av)*self.step_size).collect();
            //self.evaluator.apply_update(deltas);
            self.evaluator.update(&states[i].symmetry(), player, self.discount*target_av, self.step_size);
            self.evaluator.update(&states[i], player, self.discount*target_av, self.step_size);
        }
    }

    fn self_play(&mut self) {
        //let mut agenta = BatchMinimaxAgent::new(&*self.evaluator, self.depth, self.depth);
        let mut agenta = MinimaxPolicyAgent::new(&*self.evaluator, &*self.exploration_policy, self.depth);
        //agenta.set_player(Player::Red);
        //let mut agentb = BatchMinimaxAgent::new(&*self.evaluator, self.depth, self.depth);
        let mut agentb = MinimaxPolicyAgent::new(&*self.evaluator, &*self.exploration_policy, self.depth);
        //agentb.set_player(Player::Yellow);
        let game_hist = play_game(&agenta, &agentb);
        self.update(&game_hist, Player::Red);
        self.update(&game_hist, Player::Yellow);
    }

    fn play_against(&mut self, opponent: &dyn Agent) {
        let agent = BatchMinimaxAgent::new(&*self.evaluator, self.depth, self.depth);
        let (game_hist, selfp) = if fastrand::bool() {
            (play_game(&agent, opponent), Player::Red)
        } else {
            (play_game(opponent, &agent), Player::Yellow)
        };
        let end_state = game_hist.last().unwrap();
        if let GameState::Won(pl) = end_state.game_state {
            if pl == selfp {
                println!("won");
            } else {
                println!("lost");
            }
        }
        self.update(&game_hist, selfp);
    }

    fn get_evaluator<'a>(&'a self) -> &'a dyn Evaluator {
        &*self.evaluator
    }
}

