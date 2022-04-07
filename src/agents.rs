
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
        for action in &actions {
            board.play_action(*action);
            let v = -abnegamax(&board, self.depth-1, self.batch_depth, self.evaluator, !player, None);
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