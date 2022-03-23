
use crate::connect4::{Connect4, Player, GameState, TileStates};
use crate::connect4;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, SeqAccess};
use anyhow::Result;
use tch::nn::{ModuleT, VarStore};
use tch::TrainableCModule;
use tch::{Device};
use std::fmt;

fn pieces_in_row(board: &Connect4, pos: [usize;2], dir: [i32;2], player: Player) -> u32 {
    let mut k = 1;
    while board.in_board(pos[0] as i32+dir[0] as i32*k, pos[1] as i32+dir[1] as i32*k) 
        && board.get((pos[0] as i32+dir[0]*k) as usize, (pos[1] as i32+dir[1]*k) as usize) == player as u8 {
        k += 1;
    }
    k as u32 - 1
}

#[typetag::serde(tag = "type")]
pub trait Evaluator {

    // the estimated value at the position 'board'.
    // it must always return -infinity on loss and +infinity on win.    
    fn value(&self, board: &Connect4, player: Player) -> f64;

    // Useful when the evaluator is better at computing values in batch, for example a neural network.
    fn values(&self, boards: &Vec<Connect4>, player: Player) -> Vec<f64> {
        let mut vs = Vec::with_capacity(boards.len());
        for board in boards {
            vs.push(self.value(board, player));
        }
        vs
    }

    fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64>;
    fn apply_update(&mut self, update: &[f64]);
    //fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64);
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
                if p == player {1.0 as f64} else {-1.0 as f64}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                0.0
            },
        }
    }
    fn gradient(&self, _board: &Connect4, _player: Player) -> Vec<f64> {
        unimplemented!()
    }
    fn apply_update(&mut self, _update: &[f64]) {
        unimplemented!()
    }
    /*fn update(&mut self, _board: &Connect4, _player: Player, _target_av: f64, _learning_rate: f64) {
        unimplemented!()
    }*/
    fn get_params(&self) -> Vec<f64> {
        unimplemented!()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LinesEval {
    pub params: Vec<f64>,
}

#[typetag::serde]
impl Evaluator for LinesEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                self.lines_evaluation(board, player)
            },
        }
    }
    fn gradient(&self, _board: &Connect4, _player: Player) -> Vec<f64> {
        unimplemented!()
    }
    fn apply_update(&mut self, _update: &[f64]) {
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
                total += self.line_value(&line, player);
            }
        }
        total
    }

    fn line_value(&self, v: &Vec<TileStates>, player: Player) -> f64 {
        let mut last_opponent: i32 = -1;
        let mut count: u32 = 0;
        let mut totv = 0.0;
        for i in 0..v.len() {
            match v[i] {
                TileStates::Empty => {
                    
                },
                TileStates::Full(p) => {
                    if p != player {
                        last_opponent = i as i32;
                        count = 0;
                    } else {
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
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let features = self.features(board, player);
                let mut tot = 0.0;
                for (f, v) in features.iter().zip(self.params.iter()) {
                    tot += *f as f64*v;
                } 
                tot
            },
        }
    }
    fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        self.features(board, player)
    }
    fn apply_update(&mut self, update: &[f64]) {
        for (p, d) in self.params.iter_mut().zip(update) {
            *p += d;
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
    }
}

pub struct CNNEval {
    pub model: TrainableCModule,
    pub vs: VarStore,
}

impl CNNEval {
    pub fn new(model_path: String) -> Self {
        let device = Device::Cpu;
        let vs = VarStore::new(device);
        let model = TrainableCModule::load(&model_path, vs.root()).unwrap();
        CNNEval {
            model,
            vs,
        }
    }
    fn tmp_file_name(len: usize) -> String {
        (0..len).map(|_| fastrand::alphanumeric()).collect()
    }
}

#[typetag::serde]
impl Evaluator for CNNEval {
    fn value(&self, board: &Connect4, player: Player) -> f64 {
        match board.game_state {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let vectorized_board = board.vectorize(player);
                let tensor = unsafe {
                    let ptr = vectorized_board.as_ptr();
                    let t = tch::Tensor::of_blob(
                        ptr as *const u8, 
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
    fn values(&self, boards: &Vec<Connect4>, player: Player) -> Vec<f64> {
        let mut vectorized_boards: Vec<f64> = Vec::with_capacity(42*boards.len());
        for board in boards {
            vectorized_boards.append(&mut board.vectorize(player));
        }
        let mut tensor = tch::Tensor::of_slice(&vectorized_boards);
        let _ = tensor.resize_(&[boards.len() as i64,1,connect4::BOARD_HEIGHT as i64, connect4::BOARD_WIDTH as i64]);
        let v = self.model.forward_t(&tensor, true);
        let out: Vec<f64> = Vec::from(v);
        out
    }

    fn gradient(&self, board: &Connect4, player: Player) -> Vec<f64> {
        for var in self.vs.trainable_variables().iter_mut() {
            var.zero_grad();
        }
        let mut tboard = tch::Tensor::of_slice(&board.vectorize(player));
        let _ = tboard.resize_(&[1, 1, 6, 7]);
        let _out = self.model.forward_t(&tboard, true);
        _out.backward();
        
        let mut grad = Vec::new();
        for var in self.vs.trainable_variables().iter() {
            let mut v: Vec<f64> = Vec::from(var.grad());
            grad.append(&mut v);
        }
        grad
    }

    fn apply_update(&mut self, update: &[f64]) {
        let mut i = 0;
        for var in self.vs.trainable_variables().iter_mut() {
            let _guard = tch::no_grad_guard();
            let size = var.size();
            let n = size.iter().fold(1, |b, n| b*n);
            let mut update_tensor = tch::Tensor::of_slice(&update[i..i+n as usize]);
            let _ = update_tensor.resize_(&var.size());
            i += n as usize;
            let _ = var.f_add_(&update_tensor).unwrap();
        }
        assert_eq!(i, update.len());
    }
    /*
    fn update(&mut self, board: &Connect4, player: Player, target_av: f64, learning_rate: f64) {
        let mut optimizer = tch::nn::Sgd::default().build(&self.vs, learning_rate).unwrap();
        self.model.set_train();
        let vectorized_board = board.vectorize(player);
        let tensor = unsafe {
            let ptr = vectorized_board.as_ptr();
            let t = tch::Tensor::of_blob(
                ptr as *const u8, 
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
                ptr as *const u8, 
                &[1], 
                &[0],
                tch::Kind::Double,
                tch::Device::Cpu,
            )
        };
        let loss = out.mse_loss(&target, tch::Reduction::Mean);
        optimizer.backward_step(&loss);
    }
    */
    fn get_params(&self) -> Vec<f64> {
        unimplemented!()
    }
}

impl Serialize for CNNEval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
        where S: Serializer {
        let fname = CNNEval::tmp_file_name(10);
        self.model.save(&fname).unwrap();
        let v = std::fs::read(&fname).expect(&format!("failed to read {}", &fname));
        std::fs::remove_file(&fname).unwrap();
        serializer.serialize_bytes(&v)
    }
}

impl<'de> Deserialize<'de> for CNNEval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(ByteDataVisitor)
    }
}

struct ByteDataVisitor;
impl<'de> Visitor<'de> for ByteDataVisitor {
    type Value = CNNEval;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bytes of data")
    }

    fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error> where S: SeqAccess<'de> {
        let device = Device::Cpu;
        let vs = VarStore::new(device);
        let fname = CNNEval::tmp_file_name(10);
        let mut data = Vec::new();
        while let Some(value) = access.next_element()? {
            data.push(value);
        }
        std::fs::write(&fname, &data).expect(&format!("failed to write to {}", &fname));
        let model = TrainableCModule::load(&fname, vs.root()).expect(&format!("couldn't load module from file {}", &fname));
        std::fs::remove_file(&fname).unwrap();
        Ok(CNNEval {
            model,
            vs,
        })
    }
}
