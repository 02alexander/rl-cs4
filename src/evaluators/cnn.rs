use crate::games::{Player, GameState, Game};
use super::Evaluator;
use anyhow::Result;
use tch::nn::{ModuleT, VarStore};
use tch::TrainableCModule;
use tch::{Device};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, SeqAccess};
use std::fmt;

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

impl<G> Evaluator<G> for CNNEval where G: Game {
    fn value(&self, board: &G, player: Player) -> f64 {
        match board.game_state() {
            GameState::Won(p) => {
                if p == player {1./0.} else {-1./0.}
            },
            GameState::Draw => 0.0,
            GameState::InProgress => {
                let vectorized_board = board.vectorize(player);
                let shape = G::shape();
                let tensor = unsafe {
                    let ptr = vectorized_board.as_ptr();
                    let t = tch::Tensor::of_blob(
                        ptr as *const u8, 
                        &[1,1,shape[0] as i64, shape[1] as i64], 
                        &[0, 0, shape[0] as i64, 1],
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
    fn values(&self, boards: &Vec<G>, player: Player) -> Vec<f64> {
        let mut vectorized_boards: Vec<f64> = Vec::with_capacity(64*boards.len());
        for board in boards {
            vectorized_boards.append(&mut board.vectorize(player));
        }
        let mut tensor = tch::Tensor::of_slice(&vectorized_boards);
        let shape = G::shape();

        let _ = tensor.resize_(&[boards.len() as i64,1,shape[0] as i64, shape[1] as i64]);
        let v = self.model.forward_t(&tensor, true);
        let out: Vec<f64> = Vec::from(v);
        out
    }

    fn gradient(&self, board: &G, player: Player) -> Vec<f64> {
        for var in self.vs.trainable_variables().iter_mut() {
            var.zero_grad();
        }
        let mut tboard = tch::Tensor::of_slice(&board.vectorize(player));
        let shape = G::shape();
        let _ = tboard.resize_(&[1, 1, shape[0] as i64, shape[1] as i64]);
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
