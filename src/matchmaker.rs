
use crate::connect4::{Connect4, Player, Action, GameState};

pub trait Agent {
    fn get_action(&self, board: &Connect4, player: Player) -> Action;
}


pub struct MatchMaker<'a> {
    agents: Vec<&'a dyn Agent>, // Currently only works with 2 agents.
    hist: Vec<(usize, usize, GameState)>, // (idx of red agent, idx of yellow agent, result)
}

impl<'a> MatchMaker<'a> {
    pub fn new() -> Self {
        MatchMaker {
            agents: Vec::new(),
            hist: Vec::new(),
        }
    }

    pub fn add_agent(&mut self, agent: &'a dyn Agent) {
        self.agents.push(agent);
    }
    pub fn update_agent(&mut self, index: usize, new_agent: &'a dyn Agent) {
        self.agents[index] = new_agent
    }

    // currently only works with 2 agents.
    // plays n games between the agents and records the results in self.hist
    pub fn play_n_games(&mut self, n: u32) {
        assert_eq!(self.agents.len(), 2);
        for _ in 0..n {
            let idxagents = if fastrand::bool() {
                [0,1]
            } else {
                [1,0]
            };
            let end_board = play_game(&*self.agents[idxagents[0]], &*self.agents[idxagents[1]]);
            
            self.hist.push((idxagents[0], idxagents[1], end_board.last().unwrap().game_state));
        }
    }

    pub fn scores(&self) -> Vec<[i32;3]> {
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

    pub fn scores_hist(&self) -> Vec<Vec<[i32;3]>> {
        let mut scores = Vec::new();
        let mut current_scores = vec![[0;3];self.agents.len()]; // Vec of [draws, wins, losses]
        let mut nbgames = vec![0;self.agents.len()];
        for (pr, py, result) in &self.hist {
            nbgames[*pr] += 1;
            nbgames[*py] += 1;
            match result {
                GameState::Won(p) => {
                    match p {
                        Player::Red => {
                            current_scores[*pr][1] += 1;
                            current_scores[*py][2] += 1;
                        },
                        Player::Yellow => {
                            current_scores[*pr][2] += 1;
                            current_scores[*py][1] += 1;
                        }
                    }
                },
                GameState::Draw => {
                    current_scores[*pr][0] += 1;
                    current_scores[*py][0] += 1;
                },
                GameState::InProgress => {
                    panic!("unfinished game reached in scores_over_time");
                }
            }
            scores.push(current_scores.clone());
        }
        scores
    }
    
    pub fn avg_score_over_time(&self, agent_idx:usize, n: usize) -> Vec<f64> {
        let all_scores = self.scores_hist();
        let scores: Vec<[i32;3]> = all_scores.iter().map(|v| v[agent_idx]).collect();
        let mut r = Vec::new();
        let mut last_score = scores[agent_idx];
        for score in scores {
            if score != last_score {
                //println!("{:?}", score);
                r.push(if score[0]!=last_score[0] {
                    0.0
                } else if score[1]!=last_score[1] {
                    1.0
                } else {
                    -1.0
                });
                last_score = score;
            }
        }
        //r
        movmean(&r, n)
    }
}

fn movmean(v: &Vec<f64>, n: usize) -> Vec<f64> {
    let mut avgs = Vec::new();
    let mut sm = 0.0;
    for i in 0..v.len()-n {
        sm += v[i];
        if i >= n {
            sm -= v[i-n];
            avgs.push(sm/n as f64);
        } else {
            avgs.push(sm /(i+1) as f64);
        }
    }
    avgs
}

// returns the board after every move. which means that it excludes starting position but includes end position.
// p1 always starts.
// p1 is Player::Red, p2 is Player::Yellow.
pub fn play_game(p1: &dyn Agent, p2: &dyn Agent) -> Vec<Connect4> {
    let mut boards = Vec::new();
    let mut board = Connect4::new();

    let mut b = true;
    board.cur_player = Player::Red;
    while board.game_state == GameState::InProgress {
        let action = if b {
            p1.get_action(&board, board.cur_player)
        } else {
            p2.get_action(&board, board.cur_player)
        };
        board.play_move(action);
        b = !b;
        boards.push(board.clone());
    }
    boards
}