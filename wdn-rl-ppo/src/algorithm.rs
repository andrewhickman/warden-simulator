use std::{path::Path, time::Instant};

use burn::{
    nn::loss::{MseLoss, Reduction},
    optim::{AdamW, AdamWConfig, GradientsParams, Optimizer, adaptor::OptimizerAdaptor},
    prelude::*,
    record::{FullPrecisionSettings, JsonGzFileRecorder},
    tensor::backend::AutodiffBackend,
};
use prost::Message;
use rand::{SeedableRng, rngs::SmallRng};
use tracing::{info, warn};

use crate::{
    Config, Environment,
    model::Model,
    rollout::{Rollout, RolloutOutput},
};

pub struct Algorithm<B: AutodiffBackend, E: Environment> {
    rng: SmallRng,
    config: Config,
    model: Model<B>,
    optimizer: OptimizerAdaptor<AdamW, Model<B>, B>,
    rollout: Rollout<B, E>,
    device: B::Device,
}

impl<B: AutodiffBackend, E: Environment> Algorithm<B, E> {
    pub fn new(device: &B::Device, config: Config) -> Self {
        Self {
            rng: SmallRng::from_os_rng(),
            optimizer: AdamWConfig::new()
                .with_grad_clipping(config.clip_grad.clone())
                .init(),
            config,
            model: Model::new::<E>(device, 32),
            rollout: Rollout::new(device),
            device: device.clone(),
        }
    }

    pub fn load(&mut self, path: &Path) {
        info!("Loading model from {}", path.display());
        self.model = self
            .model
            .clone()
            .load_file(
                path,
                &JsonGzFileRecorder::<FullPrecisionSettings>::new(),
                &self.device,
            )
            .unwrap();
    }

    pub fn save(&self, path: &Path) {
        info!("Saving model to {}", path.display());
        self.model
            .clone()
            .save_file(path, &JsonGzFileRecorder::<FullPrecisionSettings>::new())
            .unwrap();
    }

    pub fn save_onnx(&self, path: &Path) {
        info!("Saving onnx to {}", path.display());
        let onnx = self.model.to_onnx().encode_to_vec();
        std::fs::write(path, onnx).unwrap();
    }

    pub fn train(&mut self, steps: usize) {
        let rollout = self.rollout.rollout(steps, &self.model);

        let start = Instant::now();

        let (mut old_policies, mut old_values) = self.model.forward(rollout.observations.clone());
        old_policies = old_policies.detach();
        old_values = old_values.detach();

        let (expected_returns, advantages) = self.get_gae(old_values, &rollout);

        for _ in 0..self.config.epochs {
            for _ in 0..(rollout.len() / self.config.batch_size) {
                let sample_indices =
                    rollout.batch_indices(&self.device, &mut self.rng, self.config.batch_size);

                let observation_batch = rollout
                    .observations
                    .clone()
                    .select(0, sample_indices.clone());
                let action_batch = rollout.actions.clone().select(0, sample_indices.clone());
                let old_policy_batch = old_policies.clone().select(0, sample_indices.clone());
                let advantage_batch = advantages.clone().select(0, sample_indices.clone());
                let expected_return_batch =
                    expected_returns.clone().select(0, sample_indices).detach();

                let (policy_batch, value_batch) = self.model.forward(observation_batch);

                let ratios = policy_batch
                    .clone()
                    .div(old_policy_batch)
                    .gather(1, action_batch);
                let clipped_ratios = ratios.clone().clamp(
                    1.0 - self.config.epsilon_clip,
                    1.0 + self.config.epsilon_clip,
                );

                let actor_loss = -elementwise_min(
                    ratios * advantage_batch.clone(),
                    clipped_ratios * advantage_batch,
                )
                .sum();
                let critic_loss =
                    MseLoss.forward(expected_return_batch, value_batch, Reduction::Sum);
                let policy_negative_entropy = -(policy_batch.clone().log() * policy_batch)
                    .sum_dim(1)
                    .mean();

                let loss = actor_loss
                    + critic_loss.mul_scalar(self.config.critic_weight)
                    + policy_negative_entropy.mul_scalar(self.config.entropy_weight);
                let gradients = loss.backward();
                let gradient_params = GradientsParams::from_grads(gradients, &self.model);

                let next_model = self.optimizer.step(
                    self.config.learning_rate,
                    self.model.clone(),
                    gradient_params,
                );

                if next_model.contains_nan() {
                    warn!("new model contains NaN values, skipping");
                    continue;
                }

                self.model = next_model;
            }
        }

        info!(
            "Finished model update after {:.3}s",
            start.elapsed().as_secs_f64()
        );
    }

    pub fn react(&self, observation: &E::Observation) -> E::Action {
        self.model.react_greedy::<E>(&self.device, observation)
    }

    fn get_gae(
        &self,
        values: Tensor<B, 2>,
        rollout: &RolloutOutput<B>,
    ) -> (Tensor<B, 2>, Tensor<B, 2>) {
        let mut returns = vec![0.0f32; rollout.len()];
        let mut advantages = vec![0.0f32; rollout.len()];

        let mut running_return: f32 = 0.0;
        let mut running_advantage: f32 = 0.0;

        for i in (0..rollout.len()).rev() {
            let reward = get_elem(i, &rollout.rewards);
            let not_done = get_elem(i, &rollout.not_dones);

            running_return = reward + self.config.gamma * running_return * not_done;
            running_advantage = reward - get_elem(i, &values)
                + self.config.gamma
                    * not_done
                    * (get_elem_or(i + 1, &values, 0.0) + self.config.lambda * running_advantage);

            returns[i] = running_return;
            advantages[i] = running_advantage;
        }

        (
            Tensor::<B, 1>::from_floats(returns.as_slice(), &self.device)
                .reshape([returns.len(), 1]),
            Tensor::<B, 1>::from_floats(advantages.as_slice(), &self.device)
                .reshape([advantages.len(), 1]),
        )
    }
}

fn elementwise_min<B: Backend, const D: usize>(
    lhs: Tensor<B, D>,
    rhs: Tensor<B, D>,
) -> Tensor<B, D> {
    let rhs_lower = rhs.clone().lower(lhs.clone());
    lhs.clone().mask_where(rhs_lower, rhs.clone())
}

fn get_elem<B: Backend, const D: usize>(i: usize, tensor: &Tensor<B, D>) -> f32 {
    tensor.to_data().as_slice().unwrap()[i]
}

fn get_elem_or<B: Backend, const D: usize>(i: usize, tensor: &Tensor<B, D>, default: f32) -> f32 {
    tensor
        .to_data()
        .as_slice()
        .unwrap()
        .get(i)
        .copied()
        .unwrap_or(default)
}
