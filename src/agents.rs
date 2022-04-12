
use crate::evaluators::{Evaluator};
use crate::search::*;
use crate::games::{Player, Game};
use crate::policies::Policy;

pub trait Agent<G> 
    where
        G: Game
{
    fn get_action(&self, board: &G, player: Player) -> G::Action;
    fn get_action_explored(&self, board: &G, player: Player) -> (G::Action, bool) {
        (self.get_action(board, player), false)
    }
}


pub struct MinimaxAgent<'a, T> {
    evaluator: &'a T,
    depth: u32,
}

impl<'a, T> MinimaxAgent<'a, T> {
    pub fn new(evaluator: &'a T, depth: u32) -> Self {
        MinimaxAgent::<T> {
            evaluator,
            depth,
        }
    }
}

impl<'a, T, G> Agent<G> for MinimaxAgent<'a, T>
    where
        G: Game,
        G::Action: Copy,
        T: Evaluator<G>
{
    fn get_action(&self, board: &G, player: Player) -> G::Action 
    {
        abnegamax_best_action(board, self.depth, self.evaluator, player)
    }
}

pub struct BatchMinimaxAgent<'a, T> {
    evaluator: &'a T,
    depth: u32,
    batch_depth: u32,
}

impl<'a, T> BatchMinimaxAgent<'a, T> {
    pub fn new(evaluator: &'a T, depth: u32, batch_depth: u32) -> Self {
        BatchMinimaxAgent::<T> {
            evaluator,
            depth,
            batch_depth,
        }
    }
}

impl<'a, T, G> Agent<G> for BatchMinimaxAgent<'a, T> 
    where
        G: Game,
        G::Action: Copy,
        T: Evaluator<G>
{
    fn get_action(&self, board: &G, player: Player) -> G::Action {
        batch_abnegamax_best_action(board, self.depth, self.batch_depth, self.evaluator, player)
    }
}


pub struct MinimaxPolicyAgent<'a, T> {
    evaluator: &'a T,
    policy: &'a dyn Policy,
    depth: u32,
    pub batch_depth: u32,
}

impl<'a, T> MinimaxPolicyAgent<'a, T> {
    pub fn new(evaluator: &'a T, policy: &'a dyn Policy, depth: u32) -> Self {
        MinimaxPolicyAgent::<T> {
            evaluator,
            policy,
            depth,
            batch_depth:0
        }
    }
}

impl<'a, T, G> Agent<G> for MinimaxPolicyAgent<'a, T> 
    where
        G: Game,
        G::Action: Copy,
        T: Evaluator<G>
{
    // If a move is guaranteed to be winning it will always be taken.
    // If a move is guaranteed to be losing it will never be taken.
    // The policy will pick among the remaining moves that are not losing.
    fn get_action(&self, board: &G, player: Player) -> G::Action {
        self.get_action_explored(board,player).0
    }

    // Returns chosen action and a boolean that is true if it was a exploring move
    fn get_action_explored(&self, board: &G, player: Player) -> (G::Action, bool) {
        let mut board = board.clone();
        let mut winning_moves = Vec::new();
        let mut avs = Vec::new();
        let actions:Vec<_> = board.legal_actions().collect();
        let mut tt = TranspositionTable::new();
        for action in &actions {
            board.play_action(*action);
            let v = -abnegamax(&board, self.depth-1, self.batch_depth, self.evaluator, !player, Some(&mut tt));
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

pub struct CompositeAgent<'a, T> {
    evaluator: &'a T,
    depth: u32,
    pub simple_depth: u32, // how deep it should search with SimpleEval.
    pub batch_depth: u32,
}

impl<'a, T> CompositeAgent<'a, T> {
    pub fn new(evaluator: &'a T, depth: u32, batch_depth: u32, simple_depth: u32) -> Self {
        CompositeAgent::<T> {
            evaluator,
            depth,
            simple_depth,
            batch_depth
        }
    }
}


impl<'a, T, G> Agent<G> for CompositeAgent<'a, T> 
    where
        G: Game,
        G::Action: Copy,
        T: Evaluator<G>
{
    // Searches at depth self.simple_depth using SimpleEval to determine losing and winning moves.
    // If there is a winning move the move will be played. 
    // If there are any moves that are unclear (first search found no win or loss for these moves) 
    // their heuristic value will be computed using self.evaluator at depth self.depth and the action
    // with maximum value will be played.
    fn get_action(&self, board: &G, player: Player) -> G::Action {
        
        let mut board = board.clone();

        let mut winning_actions = Vec::new();
        let mut losing_actions = Vec::new();
        let mut unclear_actions = Vec::new(); // actions where the search with SimpleEval returned 0.0 (heuristic value or draw).

        let actions: Vec<G::Action> = board.legal_actions().collect();
        let simple_values = vec![0.0; actions.len()];
        let mut tt = TranspositionTable::new();
        
        let simple_eval = crate::evaluators::SimpleEval::new();

        for action in actions {
            board.play_action(action);
            let v = -abnegamax(&board, self.simple_depth-1, 0, &simple_eval, !player, Some(&mut tt));
            board.reverse_last_action(action);
            if v > 0.0 {
                winning_actions.push((action, v));
            } else if v < 0.0 {
                losing_actions.push((action, v));
            } else {
                unclear_actions.push(action);
            }
        }
        if !winning_actions.is_empty() {
            winning_actions.sort_by(|(_,v1), (_,v2)| v2.partial_cmp(v1).unwrap());
            return winning_actions[0].0;
        } else if !unclear_actions.is_empty() {
            let mut tt = TranspositionTable::new();
            let mut avs = Vec::new();
            for action in unclear_actions {
                board.play_action(action);
                let v = abnegamax(&board, self.depth-1, self.batch_depth, self.evaluator, player, Some(&mut tt));
                board.reverse_last_action(action);
                avs.push((action, v));
            }
            avs.sort_by(|(_,v1), (_,v2)| v2.partial_cmp(v1).unwrap());
            return avs[0].0;
        } else if !losing_actions.is_empty() {
            losing_actions.sort_by(|(_,v1), (_,v2)| v2.partial_cmp(v1).unwrap());
            return losing_actions[0].0;
        } else {
            panic!("no action selected!")
        }
    }

    // Returns chosen action and a boolean that is true if it was a exploring move
    fn get_action_explored(&self, board: &G, player: Player) -> (G::Action, bool) {
        (self.get_action(board, player), false)
    }
}


