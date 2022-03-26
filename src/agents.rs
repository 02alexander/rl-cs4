
use crate::evaluators::{Evaluator, Evaluators};
use crate::search::*;
use crate::matchmaker::Agent;
use crate::games::connect4::{Connect4, Action};
use crate::games::{Player, Game};
use crate::policies::Policy;

pub struct MinimaxAgent<'a> {
    evaluator: &'a Evaluators,
    depth: u32,
}

impl<'a> MinimaxAgent<'a> {
    pub fn new(evaluator: &'a Evaluators, depth: u32) -> Self {
        MinimaxAgent {
            evaluator,
            depth,
        }
    }
}

impl<'a> Agent for MinimaxAgent<'a> {
    fn get_action(&self, board: &Connect4, player: Player) -> Action {
        abnegamax_best_action(board, self.depth, &*self.evaluator, player)
    }
}

pub struct BatchMinimaxAgent<'a> {
    evaluator: &'a Evaluators,
    depth: u32,
    batch_depth: u32,
}

impl<'a> BatchMinimaxAgent<'a> {
    pub fn new(evaluator: &'a Evaluators, depth: u32, batch_depth: u32) -> Self {
        BatchMinimaxAgent {
            evaluator,
            depth,
            batch_depth,
        }
    }
}

impl<'a> Agent for BatchMinimaxAgent<'a> {
    fn get_action(&self, board: &Connect4, player: Player) -> Action {
        batch_abnegamax_best_action(board, self.depth, self.batch_depth, &*self.evaluator, player)
    }
}


pub struct MinimaxPolicyAgent<'a> {
    evaluator: &'a Evaluators,
    policy: &'a dyn Policy,
    depth: u32,
    pub batch_depth: u32,
}

impl<'a> MinimaxPolicyAgent<'a> {
    pub fn new(evaluator: &'a Evaluators, policy: &'a dyn Policy, depth: u32) -> Self {
        MinimaxPolicyAgent {
            evaluator,
            policy,
            depth,
            batch_depth:depth
        }
    }
}

impl<'a> Agent for MinimaxPolicyAgent<'a> {
    // If a move is guaranteed to be winning it will always be taken.
    // If a move is guaranteed to be losing it will never be taken.
    // The policy will pick among the remaining moves that are not losing.
    fn get_action(&self, board: &Connect4, player: Player) -> Action {
        self.get_action_explored(board,player).0
    }

    // Returns chosen action and a boolean that is true if it was a exploring move
    fn get_action_explored(&self, board: &Connect4, player: Player) -> (Action, bool) {
        let mut board = board.clone();
        let mut winning_moves = Vec::new();
        let mut avs = Vec::new();
        let actions = board.legal_actions();
        for action in &actions {
            board.play_action(*action);
            let v = -batch_negamax(&board, self.depth-1, self.evaluator, !player);
            board.reverse_last_action(*action);
            if v == 1./0. {
                winning_moves.push(action);
            } else if v != -1./0. {
                avs.push((action, v));
            }
        }
        if winning_moves.len() != 0 {
            return (*winning_moves[fastrand::usize(0..winning_moves.len())], false);
        } else if avs.len() != 0 {
            let max_av = avs.iter().fold(-1./0., |b, (_,v)| v.max(b));
            let vals = avs.iter().map(|(_,v)|*v).collect();
            let i = self.policy.choose(&vals);
            return (*avs[i].0, max_av != avs[i].1);
        } else {
            return (actions[fastrand::usize(0..actions.len())], false);
        }
    }
}