use burn::{
    nn::{Initializer, Linear, LinearConfig},
    prelude::*,
    tensor::activation::{relu, softmax},
};
use bytes::Bytes;
use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
};
use wdn_rl_model::{Action, Observation};
use wdn_rl_onnx::{
    GraphProto, ModelProto, NodeProto, OperatorSetIdProto, TensorProto, TensorShapeProto,
    TypeProto, ValueInfoProto, tensor_proto::DataType, tensor_shape_proto, type_proto,
};

use crate::Environment;

#[derive(Module, Debug)]
pub(crate) struct Model<B: Backend> {
    input_size: usize,
    dense_size: usize,
    output_size: usize,
    linear: Linear<B>,
    linear_actor: Linear<B>,
    linear_critic: Linear<B>,
}

impl<B: Backend> Model<B> {
    pub(crate) fn new<E: Environment>(device: &B::Device, dense_size: usize) -> Self {
        let input_size = E::Observation::SIZE;
        let output_size = E::Action::SIZE;
        let initializer = Initializer::XavierUniform { gain: 1.0 };
        Self {
            input_size,
            dense_size,
            output_size,
            linear: LinearConfig::new(input_size, dense_size)
                .with_initializer(initializer.clone())
                .init(device),
            linear_actor: LinearConfig::new(dense_size, output_size)
                .with_initializer(initializer.clone())
                .init(device),
            linear_critic: LinearConfig::new(dense_size, 1)
                .with_initializer(initializer)
                .init(device),
        }
    }

    pub(crate) fn forward(&self, input: Tensor<B, 2>) -> (Tensor<B, 2>, Tensor<B, 2>) {
        let layer0 = relu(self.linear.forward(input));
        let policies = softmax(self.linear_actor.forward(layer0.clone()), 1);
        let values = self.linear_critic.forward(layer0);

        (policies, values)
    }

    pub(crate) fn infer(&self, input: Tensor<B, 1>) -> Tensor<B, 1> {
        let layer0 = relu(self.linear.forward(input));
        softmax(self.linear_actor.forward(layer0.clone()), 0)
    }

    pub(crate) fn react<E: Environment, R: Rng>(
        &self,
        device: &B::Device,
        observation: &E::Observation,
        rng: &mut R,
    ) -> E::Action {
        action_from_tensor(self.infer(observation_to_tensor(device, observation)), rng)
    }

    pub(crate) fn react_greedy<E: Environment>(
        &self,
        device: &B::Device,
        observation: &E::Observation,
    ) -> E::Action {
        action_from_tensor_greedy(self.infer(observation_to_tensor(device, observation)))
    }

    pub(crate) fn to_onnx(&self) -> ModelProto {
        ModelProto {
            ir_version: 8,
            opset_import: vec![OperatorSetIdProto {
                version: 18,
                ..Default::default()
            }],
            producer_name: env!("CARGO_PKG_NAME").to_owned(),
            producer_version: env!("CARGO_PKG_VERSION").to_owned(),
            graph: Some(GraphProto {
                name: "react".to_owned(),
                input: vec![ValueInfoProto {
                    name: "observation".to_owned(),
                    r#type: Some(TypeProto {
                        value: Some(type_proto::Value::TensorType(type_proto::Tensor {
                            elem_type: DataType::Float as i32,
                            shape: Some(TensorShapeProto {
                                dim: vec![
                                    tensor_shape_proto::Dimension {
                                        value: Some(
                                            tensor_shape_proto::dimension::Value::DimValue(1),
                                        ),
                                        ..Default::default()
                                    },
                                    tensor_shape_proto::Dimension {
                                        value: Some(
                                            tensor_shape_proto::dimension::Value::DimValue(
                                                self.input_size as i64,
                                            ),
                                        ),
                                        ..Default::default()
                                    },
                                ],
                            }),
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                output: vec![ValueInfoProto {
                    name: "action".to_owned(),
                    r#type: Some(TypeProto {
                        value: Some(type_proto::Value::TensorType(type_proto::Tensor {
                            elem_type: DataType::Float as i32,
                            shape: Some(TensorShapeProto {
                                dim: vec![
                                    tensor_shape_proto::Dimension {
                                        value: Some(
                                            tensor_shape_proto::dimension::Value::DimValue(1),
                                        ),
                                        ..Default::default()
                                    },
                                    tensor_shape_proto::Dimension {
                                        value: Some(
                                            tensor_shape_proto::dimension::Value::DimValue(
                                                self.output_size as i64,
                                            ),
                                        ),
                                        ..Default::default()
                                    },
                                ],
                            }),
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                initializer: vec![
                    TensorProto {
                        name: "layer0.weight".to_owned(),
                        data_type: DataType::Float as i32,
                        dims: vec![self.input_size as i64, self.dense_size as i64],
                        raw_data: to_onnx_bytes(self.linear.weight.val().to_data()),
                        ..Default::default()
                    },
                    TensorProto {
                        name: "layer0.bias".to_owned(),
                        data_type: DataType::Float as i32,
                        dims: vec![self.dense_size as i64],
                        raw_data: to_onnx_bytes(self.linear.bias.as_ref().unwrap().val().to_data()),
                        ..Default::default()
                    },
                    TensorProto {
                        name: "layer1.weight".to_owned(),
                        data_type: DataType::Float as i32,
                        dims: vec![self.dense_size as i64, self.output_size as i64],
                        raw_data: to_onnx_bytes(self.linear_actor.weight.val().to_data()),
                        ..Default::default()
                    },
                    TensorProto {
                        name: "layer1.bias".to_owned(),
                        data_type: DataType::Float as i32,
                        dims: vec![self.output_size as i64],
                        raw_data: to_onnx_bytes(
                            self.linear_actor.bias.as_ref().unwrap().val().to_data(),
                        ),
                        ..Default::default()
                    },
                ],
                node: vec![
                    NodeProto {
                        name: "layer0".to_owned(),
                        input: vec![
                            "observation".to_owned(),
                            "layer0.weight".to_owned(),
                            "layer0.bias".to_owned(),
                        ],
                        output: vec!["layer0.output".to_owned()],
                        op_type: "Gemm".to_owned(),
                        ..Default::default()
                    },
                    NodeProto {
                        name: "layer1.activate".to_owned(),
                        input: vec!["layer0.output".to_owned()],
                        output: vec!["layer1.input".to_owned()],
                        op_type: "Relu".to_owned(),
                        ..Default::default()
                    },
                    NodeProto {
                        name: "layer1".to_owned(),
                        input: vec![
                            "layer1.input".to_owned(),
                            "layer1.weight".to_owned(),
                            "layer1.bias".to_owned(),
                        ],
                        output: vec!["action".to_owned()],
                        op_type: "Gemm".to_owned(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn contains_nan(&self) -> bool {
        self.linear
            .weight
            .val()
            .contains_nan()
            .bool_or(self.linear.bias.as_ref().unwrap().val().contains_nan())
            .bool_or(self.linear_actor.weight.val().contains_nan())
            .bool_or(
                self.linear_actor
                    .bias
                    .as_ref()
                    .unwrap()
                    .val()
                    .contains_nan(),
            )
            .bool_or(self.linear_critic.weight.val().contains_nan())
            .bool_or(
                self.linear_critic
                    .bias
                    .as_ref()
                    .unwrap()
                    .val()
                    .contains_nan(),
            )
            .into_data()
            .as_slice::<bool>()
            .unwrap()[0]
    }
}

fn observation_to_tensor<B, O>(device: &B::Device, observation: &O) -> Tensor<B, 1>
where
    B: Backend,
    O: Observation,
{
    Tensor::from_floats(observation.as_array().as_ref(), device)
}

fn action_from_tensor<B, A, R>(weights: Tensor<B, 1>, rng: &mut R) -> A
where
    B: Backend,
    A: Action,
    R: Rng,
{
    let weights = weights.into_data();
    let weights = weights.as_slice::<f32>().unwrap();
    let distr = match WeightedIndex::new(weights) {
        Ok(distr) => distr,
        Err(err) => {
            panic!("invalid weights in {weights:?}: {err}");
        }
    };

    let index = distr.sample(rng);
    A::from_u32(index as u32)
}

fn action_from_tensor_greedy<B, A>(weights: Tensor<B, 1>) -> A
where
    B: Backend,
    A: Action,
{
    let weights = weights.into_data();
    A::argmax(weights.as_slice::<f32>().unwrap())
}

fn to_onnx_bytes(tensor: TensorData) -> Bytes {
    // Burn uses native endianness for tensor data, while ONNX requires little endian
    if cfg!(not(target_endian = "little")) {
        panic!("Invalid endianness");
    }

    tensor.as_bytes().to_vec().into()
}
