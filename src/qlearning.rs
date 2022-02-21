
use crate::evaluators::Evaluator;
use crate::connect4::{Connect4, Player};
use crate::policies::Policy;
use crate::matchmaker::Agent;

pub trait FuncApprox: Evaluator {
    fn gradient(&self, board: &Connect4) -> Vec<f64>;
} 

struct QLearning<T, P> {
    evaluator: T,
    exploration_policy: P, // exploration policy
    step_size: f64,
    player: Player,
}


impl<T: FuncApprox, P: Policy> QLearning<T, P> {
    pub fn new(evaluator: T, exploration_policy: P, step_size: f64, player: Player) -> QLearning<T, P> {
        QLearning {
            evaluator,
            exploration_policy,
            step_size,
            player
        }
    }

    pub fn update(&mut self, game_hist: Vec<Connect4>) {

        let mut states = Vec::with_capacity(game_hist.len()/2+1);
        for board in game_hist {
            if board.cur_player == self.player {
                states.push(board);
            }
        }
        for i in 0..(states.len()-1) {
            let grad = self.evaluator.gradient(&states[i]);
            let target = self.evaluator.value(&states[i+1]);

        }
    }
}

