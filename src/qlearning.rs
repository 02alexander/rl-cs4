
use crate::evaluators::{Evaluator, Connect4Evaluators};
use crate::games::connect4::{Connect4};
use crate::games::{GameState, Player, Game};
use crate::policies::Policy;
use crate::search::{abnegamax};
use crate::agents::{Agent, BatchMinimaxAgent, MinimaxPolicyAgent};
use serde::{Serialize, Deserialize};


// returns the board after every move. which means that it excludes starting position but includes end position.
// p1 always starts.
// p1 is Player::Red, p2 is Player::Yellow.
pub fn episode<G: Game>(p1: &dyn Agent<G>, p2: &dyn Agent<G>) -> Vec<(G, bool)> {
    let mut boards = Vec::new();
    let mut board = G::new();

    let mut b = true;
    let start_player_is_red = board.cur_player() == Player::Red;
    while board.game_state() == GameState::InProgress {
        let (action, explored) = if b == start_player_is_red {
            p1.get_action_explored(&board, board.cur_player())
        } else {
            p2.get_action_explored(&board, board.cur_player())
        };
        board.play_action(action);
        b = !b;
        boards.push((board.clone(), explored));
    }
    boards
}

pub trait RL<G, E> {

    // Update all states in game_hist where board.cur_player == 'player'.
    // game_hist is all visited position with a boolean being true if agent 
    // choosed to explore in that state.
    fn update(&mut self, game_hist: &Vec<(G, bool)>, player: Player);

    // Learns from playing against self.
    fn self_play(&mut self);

    // Learns from playing against opponent.
    fn play_against(&mut self, opponent: &dyn Agent<G>);
    
    fn get_evaluator<'a>(&'a self) -> &'a E;
    fn get_policy<'a>(&'a self) -> &'a dyn Policy;
    fn get_depth(&self) -> u32;
    fn scores(&self) -> Option<&Vec<f64>> {
        None
    }
}


#[derive(Serialize, Deserialize)]
pub struct QLearning<E> {
    pub evaluator: E,
    pub exploration_policy: Box<dyn Policy>,
    pub step_size: f64,
    pub discount: f64,
    pub depth: u32, // depth to search during training.

    // Stores scores when training against an opponent. 
    // Useful when measuring performance of algorithm.
    pub scores: Vec<f64>,

    // Decay of eligibility trace.
    pub lambda: f64, 

    eligibilty_trace: Option<Vec<f64>>,
}

impl<E> QLearning<E> { 
    pub fn new(evaluator: E, exploration_policy: Box<dyn Policy>, step_size: f64) -> Self {
        QLearning::<E> {
            evaluator,
            exploration_policy,
            step_size,
            discount: 1.0,
            depth: 4,
            scores: Vec::new(),
            lambda: 0.0, // Default is one step TD.
            eligibilty_trace: None
        }
    }
}

impl<G, E> RL<G, E> for QLearning<E> 
    where
        G: Game,
        G::Action: Copy,
        E: Evaluator<G>
{
    
    fn update(&mut self, game_hist: &Vec<(G, bool)>, player: Player) {

        // states where the board.cur_player is 'player'
        let mut states = Vec::with_capacity(game_hist.len()/2+1);
        let mut explored = Vec::with_capacity(game_hist.len()/2+1);
        for (board, e) in game_hist {
            if board.cur_player() == player {
                states.push(board);
                explored.push(*e);
            }
        }
        for i in 0..(states.len()-1) {
            let next_state = &states[i+1];
            let target_av = if i == states.len()-2 {
                match game_hist.last().unwrap().0.game_state() {
                    GameState::Won(p) => {
                        if p == player {1.0} else {-1.0}
                    },
                    GameState::Draw => { 0.0 },
                    GameState::InProgress => {
                        panic!("last state is in progress")
                    }
                }
            } else {
                let v = abnegamax(*next_state, self.depth, self.depth, &self.evaluator, player, None);

                // The reward is baked into the target action value.
                if v == 1./0. {
                    1.0
                } else if v == -1./0. {
                    -1.0
                } else {
                    v
                }
            };
            if explored[i] {
                if let Some(ref mut trace) = self.eligibilty_trace {
                    trace.iter_mut().map(|x| *x = 0.0).count();
                }
            }
            let symmetric_states = states[i].symmetries();
            //let symmetric_states = vec![states[i]];
            for state in &symmetric_states {
                let grad: Vec<f64> = self.evaluator.gradient(state, player);
                let et = self.eligibilty_trace.get_or_insert(vec![0.0;grad.len()]);
                for (trace, g) in et.iter_mut().zip(grad) {
                    *trace = self.lambda*(*trace) + g;
                }         
            }
            for state in &symmetric_states {
                let current_av = self.evaluator.value(state, player);
                let deltas: Vec<_> = self.eligibilty_trace.as_ref().unwrap().iter().map(|g| g*(self.discount*target_av-current_av)*self.step_size).collect();
                self.evaluator.apply_update(&deltas);
            }
        }
    }

    fn self_play(&mut self) {
        let agenta = MinimaxPolicyAgent::new(&self.evaluator, &*self.exploration_policy, self.depth);
        let agentb = MinimaxPolicyAgent::new(&self.evaluator, &*self.exploration_policy, self.depth);
        let game_hist: Vec<(G, bool)> = episode(&agenta, &agentb);
        self.update(&game_hist, Player::Red);
        self.update(&game_hist, Player::Yellow);
    }

    fn play_against(&mut self, opponent: &dyn Agent<G>) {
        let agent = BatchMinimaxAgent::new(&self.evaluator, self.depth, self.depth);
        let (game_hist, selfp) = if fastrand::bool() {
            (episode(&agent, opponent), Player::Red)
        } else {
            (episode(opponent, &agent), Player::Yellow)
        };
        let end_state = game_hist.last().unwrap();
        if let GameState::Won(pl) = end_state.0.game_state() {
            if pl == selfp {
                // won
                self.scores.push(1.0);
            } else {
                // lost
                self.scores.push(-1.0);
            }
        } else {
            self.scores.push(0.0);
        }
        self.update(&game_hist, selfp);
    }

    fn get_evaluator<'a>(&'a self) -> &'a E {
        &self.evaluator
    }
    fn get_policy<'a>(&'a self) -> &'a dyn Policy {
        &*self.exploration_policy
    }
    fn get_depth(&self) -> u32 {
        self.depth
    }
    fn scores(&self) -> Option<&Vec<f64>> {
        Some(&self.scores)
    }
}

