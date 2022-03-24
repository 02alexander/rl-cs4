
pub mod stack4;
pub mod connect4;

use serde::{Serialize, Deserialize};
use std::ops;

// A two player with three possible outcomes, win for either player or a draw.
pub trait Game {
    type Action;

    fn play_action(&mut self, action: Self::Action);
    fn reverse_last_action(&mut self, last_action: Self::Action);
    fn game_state(&self) -> GameState;
    fn cur_player(&self) -> Player;
    fn legal_actions(&self) -> Vec<Self::Action>;
    fn vectorize(&self, player: Player) -> Vec<f64>;
}

// in the boards these are represented by two bit numbers where Empty=0, Full(Red)=1, Full(Yellow)=2 
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum TileStates {
    Empty,
    Full(Player),
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Player {
    Red=1,
    Yellow=2,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameState {
    Won(Player),
    Draw,
    InProgress
}

impl ops::Not for Player {
    type Output = Player;
    fn not(self) -> Self {
        match self {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
        }
    }
}