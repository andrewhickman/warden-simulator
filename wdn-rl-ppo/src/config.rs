use burn::{grad_clipping::GradientClippingConfig, optim::LearningRate};

pub struct Config {
    pub gamma: f32,
    pub lambda: f32,
    pub epsilon_clip: f32,
    pub critic_weight: f32,
    pub entropy_weight: f32,
    pub learning_rate: LearningRate,
    pub epochs: usize,
    pub batch_size: usize,
    pub clip_grad: Option<GradientClippingConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gamma: 0.99,
            lambda: 0.95,
            epsilon_clip: 0.2,
            critic_weight: 0.5,
            entropy_weight: 0.01,
            learning_rate: 0.001,
            epochs: 8,
            batch_size: 8,
            clip_grad: Some(GradientClippingConfig::Value(100.0)),
        }
    }
}
