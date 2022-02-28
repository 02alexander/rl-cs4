
use serde::{Serialize, Deserialize};

pub trait Policy {
    // returns index of chosen value.
    fn choose(&self, action_values: Vec<f64>) -> usize;
}

#[derive(Serialize,Deserialize)]
pub struct EpsilonGreedy {
    epsilon: f32,
}

impl EpsilonGreedy {
    pub fn new(epsilon: f32) -> EpsilonGreedy {
        EpsilonGreedy {
            epsilon
        }
    }
}

impl Policy for EpsilonGreedy {
    fn choose(&self, action_values: Vec<f64>) -> usize {
        if fastrand::f32() < self.epsilon {
            fastrand::usize(0..action_values.len())
        } else {
            action_values.iter().enumerate().max_by(|(_,v1),(_,v2)| v1.partial_cmp(v2).unwrap()).unwrap().0
        }
    }   
}

#[derive(Serialize,Deserialize)]
pub struct Greedy {}

impl Greedy {
    pub fn new() -> Greedy {
        Greedy {}
    }
}

impl Policy for Greedy {
    fn choose(&self, action_values: Vec<f64>) -> usize {
        action_values.iter().enumerate().max_by(|(_,v1),(_,v2)| v1.partial_cmp(v2).unwrap()).unwrap().0
    }  
}