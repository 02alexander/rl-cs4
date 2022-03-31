pub mod cnn;
pub mod consequtive;
pub mod lines;
pub mod simple;

pub use consequtive::ConsequtiveEval;
pub use lines::LinesEval;
pub use simple::SimpleEval;
pub use cnn::CNNEval;
use serde::{Serialize, Deserialize};

use crate::games::{Game, Player};
use crate::games::connect4::Connect4;
use crate::games::stack4::Stack4;


pub trait Evaluator<T> where T: Game {

    // the estimated value at the position 'board'.
    // it must always return -infinity on loss and +infinity on win.    
    fn value(&self, board: &T, player: Player) -> f64;

    // Useful when the evaluator is better at computing values in batch, for example a neural network.
    fn values(&self, boards: &Vec<T>, player: Player) -> Vec<f64> {
        let mut vs = Vec::with_capacity(boards.len());
        for board in boards {
            vs.push(self.value(board, player));
        }
        vs
    }

    fn gradient(&self, board: &T, player: Player) -> Vec<f64>;
    fn apply_update(&mut self, update: &[f64]);
    //fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64);
    fn get_params(&self) -> Vec<f64>;
}

#[derive(Serialize, Deserialize)]
pub enum Stack4Evaluators {
    Simple(SimpleEval),
    Consequtive(ConsequtiveEval),
}


#[derive(Serialize, Deserialize)]
pub enum Connect4Evaluators {
    Simple(SimpleEval),
    Lines(LinesEval),
    CNN(CNNEval),
    Consequtive(ConsequtiveEval),
}

impl Evaluator<Connect4> for Connect4Evaluators {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match self {
            Connect4Evaluators::Simple(ref eval) => {eval.value(board, player)},
            Connect4Evaluators::Lines(ref eval) => {eval.value(board, player)},
            Connect4Evaluators::CNN(ref eval) => {eval.value(board, player)},
            Connect4Evaluators::Consequtive(ref eval) => {eval.value(board, player)},
        }
    }
    fn values(&self, boards: &Vec<Connect4>, player: Player) -> Vec<f64> {
        match self {
            Connect4Evaluators::Simple(ref eval) => {eval.values(boards, player)},
            Connect4Evaluators::Lines(ref eval) => {eval.values(boards, player)},
            Connect4Evaluators::CNN(ref eval) => {eval.values(boards, player)},
            Connect4Evaluators::Consequtive(ref eval) => {eval.values(boards, player)},
        }
    }
    fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        match self {
            Connect4Evaluators::Simple(ref eval) => {eval.gradient(board, player)},
            Connect4Evaluators::Lines(ref eval) => {eval.gradient(board, player)},
            Connect4Evaluators::CNN(ref eval) => {eval.gradient(board, player)},
            Connect4Evaluators::Consequtive(ref eval) => {eval.gradient(board, player)},
        }
    }
    fn apply_update(&mut self, update: &[f64]) {
        match self {
            Connect4Evaluators::Simple(ref mut eval) => {<SimpleEval as Evaluator<Connect4>>::apply_update(eval,update)},
            Connect4Evaluators::Lines(ref mut eval) => {<LinesEval as Evaluator<Connect4>>::apply_update(eval, update)},
            Connect4Evaluators::CNN(ref mut eval) => {<CNNEval as Evaluator<Connect4>>::apply_update(eval, update)},
            Connect4Evaluators::Consequtive(ref mut eval) => {<ConsequtiveEval as Evaluator<Connect4>>::apply_update(eval, update)},
        }
    }
    fn get_params(&self) -> Vec<f64> {
        match self {
            Connect4Evaluators::Simple(ref eval) => {<SimpleEval as Evaluator<Connect4>>::get_params(eval)},
            Connect4Evaluators::Lines(ref eval) => {<LinesEval as Evaluator<Connect4>>::get_params(eval)},
            Connect4Evaluators::CNN(ref eval) => {<CNNEval as Evaluator<Connect4>>::get_params(eval)},
            Connect4Evaluators::Consequtive(ref eval) => {<ConsequtiveEval as Evaluator<Connect4>>::get_params(eval)},
        }
    }
}


impl Evaluator<Stack4> for Stack4Evaluators {
    fn value(&self, board: &Stack4, player: Player) -> f64 {
        match self {
            Stack4Evaluators::Simple(ref eval) => {eval.value(board, player)},
            Stack4Evaluators::Consequtive(ref eval) => {eval.value(board, player)},
        }
    }
    fn values(&self, boards: &Vec<Stack4>, player: Player) -> Vec<f64> {
        match self {
            Stack4Evaluators::Simple(ref eval) => {eval.values(boards, player)},
            Stack4Evaluators::Consequtive(ref eval) => {eval.values(boards, player)},
        }
    }
    fn gradient(&self, board: &Stack4, player: Player) -> Vec<f64> {
        match self {
            Stack4Evaluators::Simple(ref eval) => {eval.gradient(board, player)},
            Stack4Evaluators::Consequtive(ref eval) => {eval.gradient(board, player)},
        }
    }
    fn apply_update(&mut self, update: &[f64]) {
        match self {
            Stack4Evaluators::Simple(ref mut eval) => {<SimpleEval as Evaluator<Stack4>>::apply_update(eval,update)},
            Stack4Evaluators::Consequtive(ref mut eval) => {<ConsequtiveEval as Evaluator<Stack4>>::apply_update(eval,update)},
        }
    }
    fn get_params(&self) -> Vec<f64> {
        match self {
            Stack4Evaluators::Simple(ref eval) => {<SimpleEval as Evaluator<Stack4>>::get_params(eval)},
            Stack4Evaluators::Consequtive(ref eval) => {<ConsequtiveEval as Evaluator<Stack4>>::get_params(eval)},
        }
    }
}