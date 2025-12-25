use rand::Rng;
use wdn_rl_model::{Action, Observation};

pub trait Environment: Default {
    type Action: Action;
    type Observation: Observation;

    fn reset<R: Rng>(&mut self, rng: &mut R) -> Self::Observation;
    fn step(&mut self, action: Self::Action) -> Step<Self::Observation, Self::Action>;
}

#[derive(Clone, Debug)]
pub struct Step<O, A> {
    pub observation: O,
    pub action: A,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
}

impl<O, A> Step<O, A> {
    pub fn done(&self) -> bool {
        self.terminated || self.truncated
    }
}
