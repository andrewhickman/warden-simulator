use std::{cell::RefCell, marker::PhantomData, thread::available_parallelism, time::Instant};

use burn::{prelude::*, tensor::Tensor};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use rayon::{ThreadPool, ThreadPoolBuilder};
use thread_local::ThreadLocal;
use tracing::info;
use wdn_rl_model::{Action, Observation};

use crate::{Environment, Step, model::Model};

pub(crate) struct Rollout<B: Backend, E: Environment> {
    thread_pool: ThreadPool,
    contexts: ThreadLocal<RefCell<RolloutContext<E>>>,
    device: B::Device,
    environment: PhantomData<fn() -> E>,
}

pub(crate) struct RolloutOutput<B: Backend> {
    pub len: usize,
    pub observations: Tensor<B, 2>,
    pub actions: Tensor<B, 2, Int>,
    pub rewards: Tensor<B, 2>,
    pub not_dones: Tensor<B, 2>,
}

struct RolloutContext<E: Environment> {
    rng: SmallRng,
    observations: Vec<f32>,
    actions: Vec<i32>,
    rewards: Vec<f32>,
    not_dones: Vec<f32>,
    observation: PhantomData<E::Observation>,
    len: usize,
}

impl<B: Backend, E: Environment> Rollout<B, E> {
    pub(crate) fn new(device: &B::Device) -> Self {
        let threads = available_parallelism().unwrap();
        info!("Initializing rollout thread pool with {} threads", threads);
        Self {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(threads.get())
                .build()
                .unwrap(),
            contexts: ThreadLocal::new(),
            device: device.clone(),
            environment: PhantomData,
        }
    }

    pub(crate) fn rollout(&mut self, steps: usize, model: &Model<B>) -> RolloutOutput<B> {
        let start = Instant::now();

        self.thread_pool.in_place_scope(|scope| {
            for _ in 0..self.thread_pool.current_num_threads() {
                let contexts = &self.contexts;
                let device = self.device.clone();
                let model = model.clone();
                scope.spawn(move |_| {
                    let mut env = E::default();
                    let mut context = contexts
                        .get_or(|| RefCell::new(RolloutContext::new(steps)))
                        .borrow_mut();
                    let model = model.clone();

                    context.clear();

                    let mut observation = env.reset(&mut context.rng);

                    for i in 0..steps {
                        let action = model.react::<E, _>(&device, &observation, &mut context.rng);
                        let mut result = env.step(action);

                        if result.done() {
                            observation = env.reset(&mut context.rng);
                        } else {
                            observation = result.observation.clone();
                        }

                        result.truncated = result.truncated || i == steps - 1;

                        context.push(result);
                    }
                });
            }
        });

        let len = self.contexts.iter_mut().map(|c| c.get_mut().len).sum();

        info!(
            "Completed rollout of {len} steps with average reward {} after {:.3}s",
            self.contexts
                .iter_mut()
                .flat_map(|c| &c.get_mut().rewards)
                .sum::<f32>()
                / (len as f32),
            start.elapsed().as_secs_f64(),
        );

        let observations = Tensor::cat(
            self.contexts
                .iter_mut()
                .map(|c| c.get_mut().observations(&self.device))
                .collect::<Vec<_>>(),
            0,
        );
        let actions = Tensor::cat(
            self.contexts
                .iter_mut()
                .map(|c| c.get_mut().actions(&self.device))
                .collect::<Vec<_>>(),
            0,
        )
        .unsqueeze_dim(1);
        let rewards = Tensor::cat(
            self.contexts
                .iter_mut()
                .map(|c| c.get_mut().rewards(&self.device))
                .collect::<Vec<_>>(),
            0,
        )
        .unsqueeze_dim(1);
        let not_dones = Tensor::cat(
            self.contexts
                .iter_mut()
                .map(|c| c.get_mut().not_dones(&self.device))
                .collect::<Vec<_>>(),
            0,
        )
        .unsqueeze_dim(1);

        RolloutOutput {
            len,
            observations,
            actions,
            rewards,
            not_dones,
        }
    }
}

impl<B: Backend> RolloutOutput<B> {
    pub fn batch_indices<R: Rng>(
        &self,
        device: &B::Device,
        rng: &mut R,
        batch_size: usize,
    ) -> Tensor<B, 1, Int> {
        Tensor::from_ints(
            (0..batch_size)
                .map(|_| rng.random_range(0..self.len) as i32)
                .collect::<Vec<_>>()
                .as_slice(),
            device,
        )
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<E: Environment> RolloutContext<E> {
    fn new(steps: usize) -> Self {
        Self {
            rng: SmallRng::from_os_rng(),
            observations: Vec::with_capacity(steps * E::Observation::SIZE),
            actions: Vec::with_capacity(steps),
            rewards: Vec::with_capacity(steps),
            not_dones: Vec::with_capacity(steps),
            observation: PhantomData,
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.observations.clear();
        self.actions.clear();
        self.rewards.clear();
        self.not_dones.clear();
        self.len = 0;
    }

    fn push(&mut self, step: Step<E::Observation, E::Action>) {
        step.observation.collect_into(&mut self.observations);
        self.actions.push(step.action.as_u32() as i32);
        self.rewards.push(step.reward);
        self.not_dones.push(if step.done() { 0.0 } else { 1.0 });
        self.len += 1;
    }

    fn observations<B: Backend>(&self, device: &B::Device) -> Tensor<B, 2> {
        Tensor::<B, 1>::from_floats(self.observations.as_slice(), device)
            .reshape([self.len, E::Observation::SIZE])
    }

    fn actions<B: Backend>(&self, device: &B::Device) -> Tensor<B, 1, Int> {
        Tensor::from_ints(self.actions.as_slice(), device)
    }

    fn rewards<B: Backend>(&self, device: &B::Device) -> Tensor<B, 1> {
        Tensor::from_floats(self.rewards.as_slice(), device)
    }

    fn not_dones<B: Backend>(&self, device: &B::Device) -> Tensor<B, 1> {
        Tensor::from_floats(self.not_dones.as_slice(), device)
    }
}
