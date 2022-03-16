
use crate::evaluators::Evaluator;
use crate::search::*;
use crate::matchmaker::Agent;
use crate::connect4::{Connect4, Action, Player};
use crate::policies::Policy;

pub struct MinimaxAgent<'a> {
    evaluator: &'a dyn Evaluator,
    depth: u32,
}

impl<'a> MinimaxAgent<'a> {
    pub fn new(evaluator: &'a dyn Evaluator, depth: u32) -> Self {
        MinimaxAgent {
            evaluator,
            depth,
        }
    }
}

impl<'a> Agent for MinimaxAgent<'a> {
    fn get_action(&self, board: &Connect4, player: Player) -> Action {
        abpruning_best_action(board, self.depth, &*self.evaluator, player)
    }
}

pub struct BatchMinimaxAgent<'a> {
    evaluator: &'a dyn Evaluator,
    depth: u32,
    batch_depth: u32,
}

impl<'a> BatchMinimaxAgent<'a> {
    pub fn new(evaluator: &'a dyn Evaluator, depth: u32, batch_depth: u32) -> Self {
        BatchMinimaxAgent {
            evaluator,
            depth,
            batch_depth,
        }
    }
}

impl<'a> Agent for BatchMinimaxAgent<'a> {
    fn get_action(&self, board: &Connect4, player: Player) -> Action {
        batch_negamax_best_action(board, self.depth, &*self.evaluator, player)
    }
}


pub struct MinimaxPolicyAgent<'a> {
    evaluator: &'a dyn Evaluator,
    policy: &'a dyn Policy,
    depth: u32,
    pub batch_depth: u32,
}

impl<'a> MinimaxPolicyAgent<'a> {
    pub fn new(evaluator: &'a dyn Evaluator, policy: &'a dyn Policy, depth: u32) -> Self {
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
        let mut board = board.clone();
        let mut winning_moves = Vec::new();
        let mut avs = Vec::new();
        let actions = board.valid_moves();
        for action in &actions {
            board.play_move(*action);
            let v = -batch_negamax(&board, self.depth-1, self.evaluator, !player);
            board.reverse_last_action(*action);
            if v == 1./0. {
                winning_moves.push(action);
            } else if v != -1./0. {
                avs.push((action, v));
            }
        }
        if winning_moves.len() != 0 {
            return *winning_moves[fastrand::usize(0..winning_moves.len())];
        } else if avs.len() != 0 {
            let vals = avs.iter().map(|(_,v)|*v).collect();
            let i = self.policy.choose(&vals);
            return *avs[i].0;
        } else {
            return actions[fastrand::usize(0..actions.len())];
        }
    }
}