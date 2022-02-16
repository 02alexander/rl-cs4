
use crate::connect4::{Connect4, Player, Action, GameState, TileStates};
use crate::connect4;

pub trait Agent {
    fn get_action(&self, board: &Connect4) -> Action;
    fn set_player(&mut self, player: Player);
}


pub struct MatchMaker {
    pub agents: Vec<Box<dyn Agent>>,
    hist: Vec<(usize, usize, GameState)>, // (idx of red agent, idx of yellow agent, result)
    last_game: Vec<Action>,
}

impl MatchMaker {
    pub fn new() -> Self {
        MatchMaker {
            agents: Vec::new(),
            hist: Vec::new(),
            last_game: Vec::new(),
        }
    }

    pub fn add_agent(&mut self, agent: Box<dyn Agent>) {
        self.agents.push(agent);
    }

    // currently only works with 2 agents.
    pub fn play_n_games(&mut self, n: u32) {
        assert_eq!(self.agents.len(), 2);
        for i in 0..n {
            let idxagents = if fastrand::bool() {
                [0,1]
            } else {
                [1,0]
            };
            self.agents[idxagents[0]].set_player(Player::Red);
            self.agents[idxagents[1]].set_player(Player::Yellow);
            let mut end_board = self.play_game(&self.agents[idxagents[0]], &self.agents[idxagents[1]]);
            
            self.hist.push((idxagents[0], idxagents[1], end_board.game_state));
        }
    }

    fn play_game(&self, p1: &Box<dyn Agent>, p2: &Box<dyn Agent>) -> Connect4 {
        let mut board = Connect4::new();

        let mut b = true;
        board.cur_player = Player::Red;
        while board.game_state == GameState::InProgress {
            let action = if b {
                p1.get_action(&board)
            } else {
                p2.get_action(&board)
            };
            board.play_move(action);
            b = !b;
        }
        board
    }

    pub fn get_scores(&self) -> Vec<[i32;3]> {
        let mut scores = vec![[0;3];self.agents.len()]; // Vec of [draws, wins, losses]
        let mut nbgames = vec![0;self.agents.len()];
        for (pr, py, result) in &self.hist {
            nbgames[*pr] += 1;
            nbgames[*py] += 1;
            match result {
                GameState::Won(p) => {
                    match p {
                        Player::Red => {
                            scores[*pr][1] += 1;
                            scores[*py][2] += 1;
                        },
                        Player::Yellow => {
                            scores[*pr][2] += 1;
                            scores[*py][1] += 1;
                        }
                    }
                },
                GameState::Draw => {
                    scores[*pr][0] += 1;
                    scores[*py][0] += 1;
                },
                GameState::InProgress => {
                    panic!("unfinished game reached in get_scores");
                }
            }
        }
        scores
    }
}