pub mod connect4;
pub mod stack4;

use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops;

// A two player with three possible outcomes, win for either player or a draw.
pub trait Game: Clone + Copy + fmt::Debug {
    type Action: Copy + fmt::Debug;

    fn new() -> Self;
    fn play_action(&mut self, action: Self::Action);
    fn reverse_last_action(&mut self, last_action: Self::Action);

    fn game_state(&self) -> GameState;
    fn cur_player(&self) -> Player;

    fn legal_actions(&self) -> Box<dyn Iterator<Item = Self::Action>>;

    fn vectorize(&self, player: Player) -> Vec<f64>;

    // Returns all states that are equal under symmetry including self.
    fn symmetries(&self) -> Vec<Self>;

    fn uid(&self) -> u128;

    // How many moves has been played.
    fn length(&self) -> u32;

    fn shape() -> [usize; 2];
}

// in the boards these are represented by two bit numbers where Empty=0, Full(Red)=1, Full(Yellow)=2
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum TileStates {
    Empty,
    Full(Player),
}

#[derive(Clone, Copy, PartialEq, Debug, FromPrimitive, Serialize, Deserialize)]
pub enum Player {
    Red = 1,
    Yellow = 2,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameState {
    Won(Player),
    Draw,
    InProgress,
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
