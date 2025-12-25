use anyhow::{Context, Result, bail, ensure};
use wdn_rl_onnx::NodeProto;

use crate::tensor::{ElementType, Tensor, TensorType, wrap_index, wrap_range};

#[derive(Debug, Clone)]
pub enum Var {
    Input(TensorType),
    Const(Tensor),
}

#[derive(Debug)]
pub enum Operation {
    Gemm {
        alpha: f32,
        beta: f32,
        trans_a: bool,
        trans_b: bool,
    },
    Tanh,
    Shape {
        start: i64,
        end: Option<i64>,
    },
    Constant {
        value: Tensor,
    },
    Gather {
        axis: i64,
    },
    Add,
    Div,
    Mul,
    Slice,
    Max,
    Min,
    Relu,
    Concat {
        axis: i64,
    },
}

impl Var {
    pub fn ty(&self) -> &TensorType {
        match self {
            Var::Input(tensor_type) => tensor_type,
            Var::Const(tensor) => tensor.ty(),
        }
    }

    pub fn elem_ty(&self) -> ElementType {
        self.ty().elem_ty()
    }

    pub fn shape(&self) -> &[usize] {
        self.ty().shape()
    }

    pub fn rank(&self) -> usize {
        self.ty().rank()
    }

    pub fn unwrap_const(&self) -> &Tensor {
        match self {
            Var::Const(tensor) => tensor,
            Var::Input(_) => panic!("expected var to be const"),
        }
    }
}

impl Operation {
    pub fn from_proto(node: &NodeProto) -> Result<Self> {
        match node.op_type.as_str() {
            "Gemm" => Ok(Operation::Gemm {
                alpha: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "alpha")
                    .map(|a| a.f)
                    .unwrap_or(1.0),
                beta: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "beta")
                    .map(|a| a.f)
                    .unwrap_or(1.0),
                trans_a: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "transA")
                    .map(|a| a.i != 0)
                    .unwrap_or(false),
                trans_b: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "transB")
                    .map(|a| a.i != 0)
                    .unwrap_or(false),
            }),
            "Relu" => Ok(Operation::Relu),
            "Tanh" => Ok(Operation::Tanh),
            "Shape" => Ok(Operation::Shape {
                start: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "transB")
                    .map(|a| a.i)
                    .unwrap_or(0),
                end: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "transB")
                    .map(|a| a.i),
            }),
            "Constant" => {
                let tensor = node
                    .attribute
                    .iter()
                    .find(|a| a.name == "value")
                    .and_then(|a| a.t.as_ref())
                    .context("unsupported constant type")?;
                Ok(Operation::Constant {
                    value: Tensor::from_proto(tensor)?,
                })
            }
            "Gather" => Ok(Operation::Gather {
                axis: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "axis")
                    .map(|a| a.i)
                    .unwrap_or(0),
            }),
            "Add" => Ok(Operation::Add),
            "Div" => Ok(Operation::Div),
            "Max" => Ok(Operation::Max),
            "Min" => Ok(Operation::Min),
            "Mul" => Ok(Operation::Mul),
            "Slice" => Ok(Operation::Slice),
            "Concat" => Ok(Operation::Concat {
                axis: node
                    .attribute
                    .iter()
                    .find(|a| a.name == "axis")
                    .map(|a| a.i)
                    .context("expected axis")?,
            }),
            op => bail!("unsupported operation '{op}'"),
        }
    }

    pub fn apply(&self, inputs: &[Var]) -> Result<Vec<Var>> {
        match *self {
            Operation::Add => {
                ensure!(
                    inputs.len() <= 2,
                    "invalid inputs len {} for Add operation",
                    inputs.len()
                );
                ensure!(
                    inputs[0].elem_ty() == inputs[1].elem_ty(),
                    "type mismatch for Add operation"
                );

                match (&inputs[0], &inputs[1]) {
                    (Var::Const(a), Var::Const(b)) => {
                        Ok(vec![Var::Const(Tensor::add(a.clone(), b.clone())?)])
                    }
                    (a, b) => {
                        let output_ty =
                            TensorType::multidirectional_broadcast(a.elem_ty(), &[a.ty(), b.ty()])?;

                        Ok(vec![Var::Input(output_ty)])
                    }
                }
            }
            Operation::Constant { ref value } => {
                ensure!(
                    inputs.is_empty(),
                    "invalid inputs len {} for Constant operation",
                    inputs.len()
                );

                Ok(vec![Var::Const(value.clone())])
            }
            Operation::Concat { axis } => {
                ensure!(
                    !inputs.is_empty(),
                    "invalid inputs len {} for Concat operation",
                    inputs.len()
                );

                let elem_ty = inputs[0].elem_ty();
                let rank = inputs[0].rank();
                ensure!(
                    inputs.iter().all(|i| i.elem_ty() == elem_ty),
                    "element type mismatch for Concat operation",
                );
                ensure!(
                    inputs.iter().all(|i| i.rank() == rank),
                    "rank mismatch for Concat operation",
                );

                let axis_index =
                    wrap_index(axis, rank).context("invalid axis for Concat operation")?;
                for i in 0..rank {
                    if i != axis_index {
                        let dim = inputs[0].shape()[i];
                        ensure!(
                            inputs.iter().all(|input| input.shape()[i] == dim),
                            "dimension mismatch for Concat operation",
                        );
                    }
                }

                let mut shape = inputs[0].shape().to_vec();
                shape[axis_index] = inputs.iter().map(|input| input.shape()[axis_index]).sum();

                Ok(vec![Var::Input(TensorType::new(elem_ty, shape))])
            }
            Operation::Div => {
                ensure!(
                    inputs.len() <= 2,
                    "invalid inputs len {} for Div operation",
                    inputs.len()
                );
                ensure!(
                    inputs[0].elem_ty() == inputs[1].elem_ty(),
                    "type mismatch for Div operation"
                );

                match (&inputs[0], &inputs[1]) {
                    (Var::Const(a), Var::Const(b)) => {
                        let output = Tensor::div(a.clone(), b.clone())?;
                        Ok(vec![Var::Const(output)])
                    }
                    (a, b) => {
                        let output_ty =
                            TensorType::multidirectional_broadcast(a.elem_ty(), &[a.ty(), b.ty()])?;

                        Ok(vec![Var::Input(output_ty)])
                    }
                }
            }
            Operation::Gather { axis } => {
                ensure!(
                    inputs.len() == 2,
                    "invalid inputs len {} for Gather operation",
                    inputs.len()
                );
                ensure!(
                    inputs[0].ty().dim(axis).is_some(),
                    "invalid axis {} for Gather operation on tensor of rank {}",
                    axis,
                    inputs[0].rank()
                );

                match (&inputs[0], &inputs[1]) {
                    (Var::Const(data), Var::Const(indices)) => {
                        let output = Tensor::gather(data, indices, axis)?;
                        Ok(vec![Var::Const(output)])
                    }
                    (data, indices) => {
                        let output_ty = TensorType::gather(data.ty(), indices.ty(), axis)?;
                        Ok(vec![Var::Input(output_ty)])
                    }
                }
            }
            Operation::Gemm {
                trans_a, trans_b, ..
            } => {
                ensure!(
                    2 <= inputs.len() && inputs.len() <= 3,
                    "invalid inputs len {} for Gemm operation",
                    inputs.len()
                );
                ensure!(
                    inputs[0].rank() == 2,
                    "invalid rank {} for input tensor A of Gemm operation",
                    inputs[0].rank()
                );
                ensure!(
                    inputs[1].rank() == 2,
                    "invalid rank {} for input tensor B of Gemm operation",
                    inputs[1].rank()
                );
                ensure!(
                    inputs[0].elem_ty() == inputs[1].elem_ty(),
                    "type mismatch for Gemm operation"
                );

                let elem_ty = inputs[0].elem_ty();
                let (m, k) = if trans_a {
                    (inputs[0].shape()[1], inputs[0].shape()[0])
                } else {
                    (inputs[0].shape()[0], inputs[0].shape()[1])
                };
                let (k2, n) = if trans_b {
                    (inputs[1].shape()[1], inputs[1].shape()[0])
                } else {
                    (inputs[1].shape()[0], inputs[1].shape()[1])
                };

                ensure!(
                    k == k2,
                    "dimension mismatch for Gemm operation: {:?} and {:?}",
                    inputs[0].shape(),
                    inputs[1].shape()
                );

                let output_ty = TensorType::new(elem_ty, vec![m, n]);

                if inputs.len() == 3 {
                    ensure!(
                        inputs[2].elem_ty() == elem_ty,
                        "type mismatch for Gemm operation bias"
                    );
                    if let Err(err) =
                        TensorType::unidirectional_broadcast(elem_ty, inputs[2].ty(), &output_ty)
                    {
                        bail!("invalid dimension for Gemm operation bias: {err}");
                    }
                }

                Ok(vec![Var::Input(output_ty)])
            }
            Operation::Max => {
                ensure!(
                    !inputs.is_empty(),
                    "invalid inputs len {} for Max operation",
                    inputs.len()
                );
                let elem_ty = inputs[0].elem_ty();
                ensure!(
                    inputs.iter().all(|i| i.elem_ty() == elem_ty),
                    "element type mismatch for Max operation",
                );

                let input_tys = inputs.iter().map(|var| var.ty()).collect::<Vec<_>>();
                let output_ty = TensorType::multidirectional_broadcast(elem_ty, &input_tys)?;

                Ok(vec![Var::Input(output_ty)])
            }
            Operation::Min => {
                ensure!(
                    !inputs.is_empty(),
                    "invalid inputs len {} for Min operation",
                    inputs.len()
                );
                let elem_ty = inputs[0].elem_ty();
                ensure!(
                    inputs.iter().all(|i| i.elem_ty() == elem_ty),
                    "element type mismatch for Min operation",
                );

                let input_tys = inputs.iter().map(|var| var.ty()).collect::<Vec<_>>();
                let output_ty = TensorType::multidirectional_broadcast(elem_ty, &input_tys)?;

                Ok(vec![Var::Input(output_ty)])
            }
            Operation::Mul => {
                ensure!(
                    inputs.len() <= 2,
                    "invalid inputs len {} for Mul operation",
                    inputs.len()
                );
                ensure!(
                    inputs[0].elem_ty() == inputs[1].elem_ty(),
                    "type mismatch for Mul operation"
                );

                match (&inputs[0], &inputs[1]) {
                    (Var::Const(a), Var::Const(b)) => {
                        Ok(vec![Var::Const(Tensor::mul(a.clone(), b.clone())?)])
                    }
                    (a, b) => {
                        let output_ty =
                            TensorType::multidirectional_broadcast(a.elem_ty(), &[a.ty(), b.ty()])?;

                        Ok(vec![Var::Input(output_ty)])
                    }
                }
            }
            Operation::Shape { start, end } => {
                ensure!(
                    inputs.len() == 1,
                    "invalid inputs len {} for Shape operation",
                    inputs.len()
                );

                let end = end.unwrap_or(inputs[0].rank() as i64);
                let shape = inputs[0].ty().dims(start..end)
                    .with_context(|| format!("invalid range {start}..{end} for Shape operation on tensor of shape {:?}", inputs[0].shape()))?
                    .iter()
                    .map(|&dim| dim as i64)
                    .collect::<Vec<_>>();

                let output = Tensor::from_iter_i64(
                    TensorType::new(ElementType::I64, vec![shape.len()]),
                    shape,
                );

                Ok(vec![Var::Const(output)])
            }
            Operation::Slice => {
                ensure!(
                    3 <= inputs.len() && inputs.len() <= 5,
                    "invalid inputs len {} for Slice operation",
                    inputs.len()
                );
                ensure!(
                    inputs.len() == 4,
                    "unsupported len {} for Slice operation",
                    inputs.len()
                );
                ensure!(
                    inputs[3].rank() == 1,
                    "axes must have rank 1 for Slice operation, got {:?}",
                    inputs[3].rank()
                );
                ensure!(
                    inputs[1].shape() == inputs[3].shape(),
                    "mismatch of starts and axes dimensions for Slice operation"
                );
                ensure!(
                    inputs[2].shape() == inputs[3].shape(),
                    "mismatch of ends and axes dimensions for Slice operation"
                );

                match inputs {
                    [data, Var::Const(starts), Var::Const(ends), Var::Const(axes)] => {
                        let mut shape = data.shape().to_vec();

                        for i in 0..axes.dim(0).unwrap() {
                            let start = starts.index_i64(&[i]);
                            let end = ends.index_i64(&[i]);
                            let axis = axes.index_i64(&[i]);

                            let axis_index = wrap_index(axis, data.rank())
                                .context("invalid axis in Slice operator")?;
                            let range = wrap_range(start..end, data.shape()[axis_index])
                                .context("invalid index in Slice operator")?;
                            ensure!(range.start < range.end, "invalid index in Slice operator");

                            shape[axis_index] = range.len();
                        }

                        let output_ty = TensorType::new(data.elem_ty(), shape);
                        Ok(vec![Var::Input(output_ty)])
                    }
                    _ => bail!("starts, ends and axes must all be const for Slice operation"),
                }
            }
            Operation::Tanh => {
                ensure!(
                    inputs.len() == 1,
                    "invalid inputs len {} for Tanh operation",
                    inputs.len()
                );

                Ok(vec![Var::Input(inputs[0].ty().clone())])
            }
            Operation::Relu => {
                ensure!(
                    inputs.len() == 1,
                    "invalid inputs len {} for Relu operation",
                    inputs.len()
                );

                Ok(vec![Var::Input(inputs[0].ty().clone())])
            }
        }
    }
}
