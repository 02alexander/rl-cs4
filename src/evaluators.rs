
use crate::connect4::{Connect4, Player, GameState, TileStates};
use crate::connect4;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use anyhow::Result;
use tch::nn::{Adam, ModuleT, OptimizerConfig, VarStore};
use tch::vision::dataset::Dataset;
use tch::TrainableCModule;
use tch::{CModule, Device};
use std::fmt;

#[typetag::serde(tag = "type")]
pub trait Evaluator {
    fn value(&self, board: &Connect4, player: Player) -> f64;
    //fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64>;
    //fn apply_update(&mut self, change: Vec<f64>);
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64);
    fn get_params(&self) -> Vec<f64>;
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SimpleEval {
}


impl SimpleEval {
    pub fn new() -> SimpleEval {
        SimpleEval {}
    }
}

#[typetag::serde]
impl Evaluator for SimpleEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                //if p == self.player {1./0.} else {-1./0.}
                if p == player {1.0 as f64} else {-1.0 as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                0.0
            },
        }
    }
    /*fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        unimplemented!()
    }
    fn apply_update(&mut self, change: Vec<f64>) {
        unimplemented!()
    }*/
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64) {
        unimplemented!()
    }
    fn get_params(&self) -> Vec<f64> {
        unimplemented!()
    }
}

fn pieces_in_row(board: &Connect4, pos: [usize;2], dir: [i32;2], player: Player) -> u32 {
    let mut k = 1;
    while board.in_board(pos[0] as i32+dir[0] as i32*k, pos[1] as i32+dir[1] as i32*k) 
        && board.get((pos[0] as i32+dir[0]*k) as usize, (pos[1] as i32+dir[1]*k) as usize) == player as u8 {
        k += 1;
    }
    k as u32 - 1
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LinesEval {
    pub params: Vec<f64>,
}

#[typetag::serde]
impl Evaluator for LinesEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        //println!("LinesEval.value()");
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
                //if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                self.lines_evaluation(board, player)
            },
        }
    }
    /*fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        unimplemented!()
    }
    fn apply_update(&mut self, change: Vec<f64>) {
        unimplemented!()
    }*/
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64) {
        unimplemented!()
    }
    fn get_params(&self) -> Vec<f64> {
        self.params.clone()
    }
}

impl LinesEval {

    pub fn new() -> LinesEval {
        LinesEval {
            params: vec![0.0, 0.0], // [ v for 2 in a row, v for 3 in a row]
        }
    }

    fn lines_evaluation(&self, board: &Connect4, player: Player) -> f64 {
        let mut total = 0.0;
        let directions = [[1, 0],[0, 1], [1,1]];
        //iterates over the first row and the first column.
        for (r,c) in (0..connect4::BOARD_HEIGHT).map(|r|(r,0)).chain((0..connect4::BOARD_HEIGHT).map(|c|(0,c))) {
            //println!("{:?}", (r,c));
            for dir in directions {
                let mut line = Vec::with_capacity(connect4::BOARD_WIDTH); 
                let mut k = 0;
                loop {
                    if !board.in_board(k*dir[1]+c as i32, k*dir[0]+r as i32) {
                        break
                    }
                    line.push(match board.get((k*dir[1]+c as i32) as usize, (k*dir[0]+r as i32) as usize) {
                        0 => TileStates::Empty,
                        1 => TileStates::Full(Player::Red),
                        _ => TileStates::Full(Player::Yellow)
                    });
                    k += 1;
                }
                //println!("line={:?}", line);
                total += self.line_value(&line, player);
            }
        }
        total
    }

    fn line_value(&self, v: &Vec<TileStates>, player: Player) -> f64 {
        let mut last_opponent: i32 = -1;
        let mut count: u32 = 0;
        let mut totv = 0.0;
        //println!("{:?}", v);
        for i in 0..v.len() {
            match v[i] {
                TileStates::Empty => {
                    
                },
                TileStates::Full(p) => {
                    if p != player {
                        last_opponent = i as i32;
                        count = 0;
                    } else {
                        //println!("added to count");
                        count += 1;
                    }
                }
            }
            if i as i32-last_opponent >= 4 {
                totv += count as f64;
            }
        }
        totv
    }

}


#[derive(Clone, Serialize, Deserialize)]
pub struct ConsequtiveEval {
    pub params: Vec<f64>,
}

#[typetag::serde]
impl Evaluator for ConsequtiveEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        //println!("ConsequtiveEval.value()");
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
                //if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                //self.lines_evaluation(board)
                let features = self.features(board, player);
                let mut tot = 0.0;
                for (f, v) in features.iter().zip(self.params.iter()) {
                    tot += *f as f64*v;
                } 
                //println!("{:?}", tot);
                tot
            },
        }
    }
    /*fn apply_update(&mut self, change: Vec<f64>) {
        for i in 0..change.len() {
            self.params[i] += change[i];
        }
    }
    fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        self.features(board, player)
    }*/
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64) {
        let features = self.features(board, player);
        let mut av = 0.0;
        for (f, v) in features.iter().zip(self.params.iter()) {
            av += *f as f64*v;
        } 
        let deltas: Vec<f64> = features.iter().map(|g| g*(target_av-av)*learning_rate).collect();
        for i in 0..deltas.len() {
            self.params[i] += deltas[i];
        }
    }
    fn get_params(&self) -> Vec<f64> {
        self.params.clone()
    }
}

impl ConsequtiveEval {

    pub fn new() -> Self {
        ConsequtiveEval {
            params: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // [v for 1 in a row,  v for 2 in a row, v for 3 in a row,    for opponent]
        }
    }

    fn features(&self, board: &Connect4, player: Player) -> Vec<f64> {
        let directions = [[1,0], [1,1], [0,1], [-1, 1]];
        let mut f = vec![0;6];
        for x in 0..connect4::BOARD_WIDTH {
            for y in 0..connect4::BOARD_HEIGHT {
                if board.get(x,y) != 0 {
                    continue
                }
                for dir in directions {
                    let a = pieces_in_row(board, [x,y], dir, player);
                    let b = pieces_in_row(board, [x,y], [-dir[0], -dir[1]], player);
                    let l = 3.min(a+b);
                    /*if l == 3 {
                        println!("{:?} {:?}, {}, {}", [x,y], dir, a,b);
                    }*/
                    if l >= 1 {
                        f[l as usize-1] += 1;
                    }
                }
            }
        }
        for x in 0..connect4::BOARD_WIDTH {
            for y in 0..connect4::BOARD_HEIGHT {
                if board.get(x,y) != 0 {
                    continue
                }
                for dir in directions {
                    let a = pieces_in_row(board, [x,y], dir, !player);
                    let b = pieces_in_row(board, [x,y], [-dir[0], -dir[1]], !player);
                    let l = 3.min(a+b);
                    if l >= 1 {
                        f[l as usize-1+3] += 1;
                    }
                }
            }
        }
        let mx = 10.0;
        f.iter().map(|x| mx*(1.0-(-x as f64/mx).exp())).collect()
        //f.iter().map(|x| *x as f64).collect()
    }
}


/*
#[derive(Clone, Serialize, Deserialize)]
pub struct CNNEval {
    
}

#[typetag::serde]
impl Evaluator for CNNEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        //println!("ConsequtiveEval.value()");
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
                //if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let r: PyResult<f64> = Python::with_gil(|py| {
                    let ceval = PyModule::from_code(py, r#"
ceval
                    "#, "", "").unwrap();
                    
                    Ok(2.0)
                });
                r.unwrap()
            },
        }
    }
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64) {
        Python::with_gil(|py| {
            let ceval = PyModule::from_code(py, "
            ", "", "").unwrap();
        });
    }
    fn get_params(&self) -> Vec<f64> {
        unimplemented!()
    }
}

impl CNNEval {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            py.run(r#"
import cnneval
ceval = CNNEval()
def update(state, target_av, learning_rate):
    global ceval
    ceval.update(state, target_av, learning_rate)
            "#, None, None).unwrap();  
        });
        CNNEval {
            
        }
    }
}*/


pub struct CNNEval {
    pub model_path: String,
    pub model: TrainableCModule,
    pub vs: VarStore,
}

impl CNNEval {
    pub fn new(model_path: String) -> Self {
        let device = Device::Cpu;
        let vs = VarStore::new(device);
        let model = TrainableCModule::load(&model_path, vs.root()).unwrap();
        let opt = tch::nn::sgd(0.0, 0.0, 0.0, false);
        CNNEval {
            model_path,
            model,
            vs,
        }
    }
}

impl Serialize for CNNEval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
        where S: Serializer {
        self.model.save(&self.model_path).unwrap();
        println!("serialize called");
        serializer.serialize_str(&self.model_path)

    }
}

impl<'de> Deserialize<'de> for CNNEval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_string(ModelPathVisitor)
    }
}

struct ModelPathVisitor;
impl<'de> Visitor<'de> for ModelPathVisitor {
    type Value = CNNEval;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string to the model")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> where E: de::Error {
        let device = Device::Cpu;
        let vs = VarStore::new(device);
        let model_path = String::from(s);
        let model = TrainableCModule::load(&model_path, vs.root()).expect(&format!("couldn't load module from file {}", &model_path));
        Ok(CNNEval {
            model_path,
            model,
            vs,
        })
    }
}

#[typetag::serde]
impl Evaluator for CNNEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
                //if p == self.player {1.0/board.actions.len() as f64} else {-1.0/board.actions.len() as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let vectorized_board = board.vectorize(player);
                let tensor = unsafe {
                    let ptr = vectorized_board.as_ptr();
                    let t = tch::Tensor::of_blob(
                        (ptr as *const u8), 
                        &[1,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64], 
                        &[0, 0, connect4::BOARD_WIDTH as i64, 1],
                        tch::Kind::Double,
                        tch::Device::Cpu,
                    );
                    t
                };
                let v = self.model.forward_t(&tensor, true);
                let data_ptr = v.data_ptr();
                unsafe {
                    *(data_ptr as *const f64)
                }
            },
        }
    }
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64) {
        let mut optimizer = tch::nn::Sgd::default().build(&self.vs, learning_rate).unwrap();
        self.model.set_train();
        let vectorized_board = board.vectorize(player);
        let tensor = unsafe {
            let ptr = vectorized_board.as_ptr();
            let t = tch::Tensor::of_blob(
                (ptr as *const u8), 
                &[1,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64], 
                &[0, 0, connect4::BOARD_WIDTH as i64, 1],
                tch::Kind::Double,
                tch::Device::Cpu,
            );
            t
        };
        let out = self.model.forward_t(&tensor, true);
        let targetv = vec![target_av];
        let target = unsafe {
            let ptr = targetv.as_ptr();
            tch::Tensor::of_blob(
                (ptr as *const u8), 
                &[1], 
                &[0],
                tch::Kind::Double,
                tch::Device::Cpu,
            )
        };
        let loss = out.mse_loss(&target, tch::Reduction::Mean);
        optimizer.backward_step(&loss);
    }
    fn get_params(&self) -> Vec<f64> {
        unimplemented!()
    }
}