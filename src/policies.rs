
use serde::{Serialize, Deserialize};

#[typetag::serde(tag = "type")]
pub trait Policy {
    // returns index of chosen value.
    fn choose(&self, action_values: &Vec<f64>) -> usize;
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

#[typetag::serde]
impl Policy for EpsilonGreedy {
    fn choose(&self, action_values: &Vec<f64>) -> usize {
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

#[typetag::serde]
impl Policy for Greedy {
    fn choose(&self, action_values: &Vec<f64>) -> usize {
        let mx = action_values.iter().fold(-1./0. as f64, |a, &v| a.max(v));
        let best_actions: Vec<(usize,f64)> = action_values.iter().enumerate().filter(|(_,v)| **v==mx).map(|(i,v)| (i,*v)).collect();
        return best_actions[fastrand::usize(0..best_actions.len())].0;
    }  
}