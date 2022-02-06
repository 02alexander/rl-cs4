mod connect4;
mod minimax;
mod evaluators;

use connect4::{Connect4, Player, BOARD_HEIGHT, BOARD_WIDTH, GameState};
use std::io::{self, BufRead};


fn main() {
    user_vs_user()
}

fn user_vs_user() {
    let mut stdin = io::stdin();
    let mut board = Connect4::new();
    let mut cur_player = Player::Red;

    println!("{:?}", board);
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        
        let action = line.parse::<usize>();
        if let Ok(a) = action {
            if a >= 0 && a < connect4::BOARD_WIDTH {
                
                if !board.is_valid_move(a) {
                    println!("Column alread full");
                    continue;
                }
                
                board.play_move(a, cur_player);
                println!("{:?}", board);
                println!("{:?}", board.game_state);
                match board.game_state {
                    GameState::Draw => {
                        println!("Draw");
                    }
                    GameState::InProgress => {},
                    GameState::Won(player) => {
                        println!("{:?} won", player);   
                    }
                }

                cur_player = !cur_player;

            } else {
                println!("Not in range 0..{}", connect4::BOARD_WIDTH);    
            }
        } else {
            println!("Invalid input: try again");
        }
        
    }

}