
use crate::evaluators::{Evaluator, ConsequtiveEval, LinesEval};
use crate::connect4::{Connect4, Player, N_ACTIONS, Action};
use crate::policies::Policy;
use crate::matchmaker::{Agent, play_game};
use crate::search::{abpruning_value};
use crate::search::MinimaxAgent;
use serde::{Serialize, Deserialize};

pub trait FuncApprox: Evaluator {
    fn gradient(&self, board: &Connect4) -> Vec<f64>;
    fn apply_update(&mut self, change: Vec<f64>);
} 

#[derive(Serialize, Deserialize)]
pub struct QLearning<T, P> {
    evaluator: T,
    exploration_policy: P, // exploration policy
    step_size: f64,
    pub discount: f64,
    pub depth: u32, // depth to search during training.
}

pub struct PolicyMiniMaxAgent<P> {
    exploration_policy: P,
    depth: u32
}

impl<P: Policy> PolicyMiniMaxAgent<P> {
    pub fn new(exploration_policy: P, depth: u32) -> Self {
        PolicyMiniMaxAgent {
            exploration_policy,
            depth
        }
    }
}

impl<P> Agent for PolicyMiniMaxAgent<P> {
    fn set_player(&mut self, player: Player) {

    }
    fn get_action(&self, board: &Connect4) -> Action {
        unimplemented!()
    }
}

impl<T: FuncApprox+Clone, P: Policy> QLearning<T, P> {
    pub fn new(evaluator: T, exploration_policy: P, step_size: f64) -> QLearning<T, P> {
        QLearning {
            evaluator,
            exploration_policy,
            step_size,
            discount: 1.0,
            depth: 4,
        }
    }

    pub fn update(&mut self, game_hist: &Vec<Connect4>, player: Player) {

        let mut states = Vec::with_capacity(game_hist.len()/2+1);
        for board in game_hist {
            if board.cur_player == player {
                states.push(board);
            }
        }
        for i in 0..(states.len()-1) {
            let grad = self.evaluator.gradient(&states[i]);
            
            let mut next_state = states[i+1].clone();
            let actions = next_state.valid_moves();
            let target_av = if i == states.len()-2 {
                self.evaluator.value(&next_state)
            } else {
                /*let mut max_av = -1.0/0.;
                for action in actions {
                    next_state.play_move(action);
                    let av = self.evaluator.value(&next_state);
                    next_state.reverse_last_move();
                    if av > max_av {
                        max_av = av;
                    }
                }
                max_av
                */
                abpruning_value(&next_state, self.depth, &self.evaluator)
            };
            //println!("{:.5}", target_av);
            let current_av = self.evaluator.value(&states[i]);
            let deltas = grad.iter().map(|g| g*(self.discount*target_av-current_av)*self.step_size).collect();
            self.evaluator.apply_update(deltas);
        }
    }

    pub fn self_play(&mut self) {
        let mut agenta = MinimaxAgent::new(self.evaluator.clone(), self.depth);
        agenta.set_player(Player::Red);
        let mut agentb = MinimaxAgent::new(self.evaluator.clone(), self.depth);
        agentb.set_player(Player::Yellow);
        let game_hist = play_game(&agenta, &agentb);
        self.update(&game_hist, Player::Red);
        self.update(&game_hist, Player::Yellow);
    }

    pub fn play_against<A: Agent>(&mut self, opponent: &A, opponent_player: Player) {
        let mut agent = MinimaxAgent::new(self.evaluator.clone(), self.depth);
        agent.set_player(!opponent_player);
        let game_hist = if fastrand::bool() {
            play_game(&agent, opponent)
        } else {
            play_game(opponent, &agent)
        };
        self.update(&game_hist, !opponent_player);
    }

    pub fn get_evaluator(&self) -> &T {
        &self.evaluator
    }
}

