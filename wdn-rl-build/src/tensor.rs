use std::{cmp, ops::Range};

use anyhow::{Context, Result, bail, ensure};
use itertools::Itertools;
use prost::bytes::{Buf, BufMut, Bytes, BytesMut};

use wdn_rl_onnx::{self as onnx, tensor_proto};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ElementType {
    F32,
    I64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TensorType {
    elem_ty: ElementType,
    shape: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    ty: TensorType,
    data: Bytes,
}

impl ElementType {
    pub fn from_proto(ty: tensor_proto::DataType) -> Result<Self> {
        match ty {
            tensor_proto::DataType::Float => Ok(ElementType::F32),
            tensor_proto::DataType::Int64 => Ok(ElementType::I64),
            _ => bail!("unsupported tensor type: {ty:?}"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ElementType::F32 => 4,
            ElementType::I64 => 8,
        }
    }
}

impl TensorType {
    pub fn new(elem_ty: ElementType, shape: Vec<usize>) -> Self {
        TensorType { elem_ty, shape }
    }

    pub fn from_tensor_proto(tensor: &onnx::TensorProto) -> Result<Self> {
        let elem_ty = ElementType::from_proto(tensor.data_type.try_into()?)?;
        let shape = tensor
            .dims
            .iter()
            .map(|&dim| Ok(usize::try_from(dim)?))
            .collect::<Result<Vec<_>>>()?;

        Ok(TensorType { elem_ty, shape })
    }

    pub fn from_value_info_proto(value_info: &onnx::ValueInfoProto) -> Result<Self> {
        let ty = value_info
            .r#type
            .as_ref()
            .context("expected type in ValueInfo")?;
        match &ty.value {
            Some(onnx::type_proto::Value::TensorType(tensor_ty)) => {
                let elem_ty = ElementType::from_proto(tensor_ty.elem_type.try_into()?)?;

                let shape = tensor_ty
                    .shape
                    .as_ref()
                    .context("expected shape in TypeProto.Tensor")?
                    .dim
                    .iter()
                    .map(|dim| match &dim.value {
                        &Some(onnx::tensor_shape_proto::dimension::Value::DimValue(value)) => {
                            Ok(usize::try_from(value)?)
                        }
                        Some(onnx::tensor_shape_proto::dimension::Value::DimParam(param)) => {
                            bail!("dimension param '{param}' not supported")
                        }
                        None => bail!("expected dimension"),
                    })
                    .collect::<Result<Vec<_>>>()
                    .context("invalid dimension in TypeProto.Tensor")?;

                Ok(TensorType { elem_ty, shape })
            }
            Some(value) => bail!("unsupported value in ValueInfo: {:?}", value),
            None => bail!("expected type value in ValueInfo"),
        }
    }

    pub fn unidirectional_broadcast(
        elem_ty: ElementType,
        a: &TensorType,
        b: &TensorType,
    ) -> Result<TensorType> {
        let rank = cmp::max(a.rank(), b.rank()) as i64;
        let shape = (0..rank)
            .rev()
            .map(|i| match (a.dim(-i), b.dim(-i)) {
                (Some(a), Some(1) | None) => Ok(a),
                (Some(a), Some(b)) if a == b => Ok(a),
                _ => bail!(
                    "incompatible dimensions for unidirectional broadcast: {:?} and {:?}",
                    a.shape(),
                    b.shape()
                ),
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(TensorType::new(elem_ty, shape))
    }

    pub fn multidirectional_broadcast(
        elem_ty: ElementType,
        tys: &[&TensorType],
    ) -> Result<TensorType> {
        let rank = tys
            .iter()
            .map(|t| t.rank())
            .max()
            .context("expected at least one tensor")?;
        let shape = (0..rank)
            .map(|i| {
                tys.iter()
                    .map(|ty| ty.dim(i as i64))
                    .try_fold(1, |a, b| match (a, b) {
                        (a, Some(1) | None) => Ok(a),
                        (1, Some(b)) => Ok(b),
                        (a, Some(b)) if a == b => Ok(a),
                        _ => bail!("incompatible dimensions for multidirectional broadcast"),
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(TensorType::new(elem_ty, shape))
    }

    pub fn elem_ty(&self) -> ElementType {
        self.elem_ty
    }

    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    pub fn len(&self) -> usize {
        self.elem_ty.len() * self.shape.iter().product::<usize>()
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    pub fn dim(&self, axis: i64) -> Option<usize> {
        wrap_index(axis, self.rank()).map(|axis| self.shape[axis])
    }

    pub fn dims(&self, range: Range<i64>) -> Option<&[usize]> {
        wrap_range(range, self.rank()).map(|range| &self.shape[range])
    }

    pub fn indices(&self) -> impl Iterator<Item = Vec<usize>> {
        self.shape.iter().map(|&i| 0..i).multi_cartesian_product()
    }

    pub fn gather(data: &TensorType, indices: &TensorType, axis: i64) -> Result<TensorType> {
        let axis_index = wrap_index(axis, data.rank()).context("invalid axis")?;

        let mut shape = data.shape().to_vec();
        shape
            .splice(
                axis_index..(axis_index + 1),
                indices.shape().iter().copied(),
            )
            .for_each(drop);

        Ok(TensorType::new(data.elem_ty(), shape))
    }
}

impl Tensor {
    pub fn new(ty: TensorType, data: Bytes) -> Result<Self> {
        ensure!(
            data.len() == ty.len(),
            "invalid data len {} for type {:?}",
            data.len(),
            ty,
        );

        Ok(Tensor { ty, data })
    }

    pub fn from_proto(tensor: &onnx::TensorProto) -> Result<Self> {
        ensure!(!tensor.raw_data.is_empty(), "only raw data is supported");
        ensure!(tensor.segment.is_none(), "segmented format not supported");
        ensure!(
            tensor.external_data.is_empty(),
            "external data not supported"
        );

        let shape = TensorType::from_tensor_proto(tensor)?;
        let data = tensor.raw_data.clone();

        Tensor::new(shape, data)
    }

    pub fn from_fn_f32(ty: TensorType, f: impl Fn(&[usize]) -> f32) -> Self {
        Tensor::from_iter_f32(ty.clone(), ty.indices().map(|indices| f(&indices)))
    }

    pub fn from_fn_i64(ty: TensorType, f: impl Fn(&[usize]) -> i64) -> Self {
        Tensor::from_iter_i64(ty.clone(), ty.indices().map(|indices| f(&indices)))
    }

    pub fn from_iter_f32(ty: TensorType, iter: impl IntoIterator<Item = f32>) -> Self {
        let mut data = BytesMut::with_capacity(ty.len());
        for item in iter {
            data.put_f32_le(item);
        }

        assert_eq!(data.len(), ty.len());
        Tensor {
            ty,
            data: data.freeze(),
        }
    }

    pub fn from_iter_i64(ty: TensorType, iter: impl IntoIterator<Item = i64>) -> Self {
        let mut data = BytesMut::with_capacity(ty.len());
        for item in iter {
            data.put_i64_le(item);
        }

        assert_eq!(data.len(), ty.len());
        Tensor {
            ty,
            data: data.freeze(),
        }
    }

    pub fn elem_ty(&self) -> ElementType {
        self.ty.elem_ty
    }

    pub fn shape(&self) -> &[usize] {
        &self.ty.shape
    }

    pub fn ty(&self) -> &TensorType {
        &self.ty
    }

    pub fn rank(&self) -> usize {
        self.shape().len()
    }

    pub fn dim(&self, axis: i64) -> Option<usize> {
        self.ty().dim(axis)
    }

    pub fn index_f32(&self, indices: &[usize]) -> f32 {
        assert_eq!(self.elem_ty(), ElementType::F32);
        self.index_raw(indices).get_f32_le()
    }

    pub fn index_i64(&self, indices: &[usize]) -> i64 {
        assert_eq!(self.elem_ty(), ElementType::I64);
        self.index_raw(indices).get_i64_le()
    }

    fn index_raw(&self, indices: &[usize]) -> &[u8] {
        assert!(self.rank() >= indices.len());

        let mut stride = self.elem_ty().len();
        let mut i = 0;
        for axis in (0..self.rank()).rev() {
            match self.shape().get(axis) {
                None | Some(1) => (),
                Some(&dim) => {
                    i += indices[axis] * stride;
                    stride *= dim;
                }
            }
        }

        &self.data[i..][..self.elem_ty().len()]
    }

    pub fn iter_f32(&self) -> impl Iterator<Item = f32> {
        assert_eq!(self.elem_ty(), ElementType::F32);
        self.data
            .chunks(ElementType::F32.len())
            .map(|mut chunk| chunk.get_f32_le())
    }

    pub fn iter_i64(&self) -> impl Iterator<Item = i64> {
        assert_eq!(self.elem_ty(), ElementType::I64);
        self.data
            .chunks(ElementType::I64.len())
            .map(|mut chunk| chunk.get_i64_le())
    }

    pub fn add(a: Tensor, b: Tensor) -> Result<Tensor> {
        ensure!(
            a.elem_ty() == b.elem_ty(),
            "element type mismatch for add operation"
        );

        let elem_ty = a.elem_ty();
        let ty = TensorType::multidirectional_broadcast(elem_ty, &[a.ty(), b.ty()])?;
        match elem_ty {
            ElementType::F32 => Ok(Tensor::from_fn_f32(ty, |indices| {
                a.index_f32(indices) + b.index_f32(indices)
            })),
            ElementType::I64 => Ok(Tensor::from_fn_i64(ty, |indices| {
                a.index_i64(indices) + b.index_i64(indices)
            })),
        }
    }

    pub fn div(a: Tensor, b: Tensor) -> Result<Tensor> {
        ensure!(
            a.elem_ty() == b.elem_ty(),
            "element type mismatch for div operation"
        );

        let elem_ty = a.elem_ty();
        let ty = TensorType::multidirectional_broadcast(elem_ty, &[a.ty(), b.ty()])?;
        match elem_ty {
            ElementType::F32 => Ok(Tensor::from_fn_f32(ty, |indices| {
                a.index_f32(indices) / b.index_f32(indices)
            })),
            ElementType::I64 => Ok(Tensor::from_fn_i64(ty, |indices| {
                a.index_i64(indices) / b.index_i64(indices)
            })),
        }
    }

    pub fn mul(a: Tensor, b: Tensor) -> Result<Tensor> {
        ensure!(
            a.elem_ty() == b.elem_ty(),
            "element type mismatch for mul operation"
        );

        let elem_ty = a.elem_ty();
        let ty = TensorType::multidirectional_broadcast(elem_ty, &[a.ty(), b.ty()])?;
        match elem_ty {
            ElementType::F32 => Ok(Tensor::from_fn_f32(ty, |indices| {
                a.index_f32(indices) * b.index_f32(indices)
            })),
            ElementType::I64 => Ok(Tensor::from_fn_i64(ty, |indices| {
                a.index_i64(indices) * b.index_i64(indices)
            })),
        }
    }

    pub fn gather(data: &Tensor, indices: &Tensor, axis: i64) -> Result<Tensor> {
        let ty = TensorType::gather(data.ty(), indices.ty(), axis)?;
        let axis_start = wrap_index(axis, ty.rank()).context("invalid axis")?;
        let axis_end = axis_start + indices.rank();

        let data_indices = ty.indices().map(|mut data_indices| {
            let axis_index = wrap_index(
                indices.index_i64(&data_indices[axis_start..axis_end]),
                data.shape()[axis_start],
            )
            .expect("invalid index in Gather");
            data_indices
                .splice(axis_start..axis_end, [axis_index])
                .for_each(drop);
            data_indices
        });

        match ty.elem_ty() {
            ElementType::F32 => Ok(Tensor::from_iter_f32(
                ty.clone(),
                data_indices.map(|indices| data.index_f32(&indices)),
            )),
            ElementType::I64 => Ok(Tensor::from_iter_i64(
                ty.clone(),
                data_indices.map(|indices| data.index_i64(&indices)),
            )),
        }
    }
}

pub fn wrap_range(range: Range<i64>, len: usize) -> Option<Range<usize>> {
    match (wrap_index(range.start, len), wrap_end_index(range.end, len)) {
        (Some(start), Some(end)) => Some(start..end),
        _ => None,
    }
}

pub fn wrap_index(index: i64, len: usize) -> Option<usize> {
    if 0 <= index && index < len as i64 {
        Some(index as usize)
    } else if -(len as i64) <= index && index < 0 {
        Some((len as i64 + index) as usize)
    } else {
        None
    }
}

fn wrap_end_index(index: i64, len: usize) -> Option<usize> {
    if 0 <= index && index <= len as i64 {
        Some(index as usize)
    } else if -(len as i64) <= index && index < 0 {
        Some((len as i64 + index) as usize)
    } else {
        None
    }
}

#[test]
fn gather() {
    let data = Tensor::from_iter_f32(
        TensorType::new(ElementType::F32, vec![3, 3]),
        [[1.0, 1.2, 1.9], [2.3, 3.4, 3.9], [4.5, 5.7, 5.9]]
            .into_iter()
            .flatten(),
    );
    let indices = Tensor::from_iter_i64(
        TensorType::new(ElementType::I64, vec![1, 2]),
        [[0, 2]].into_iter().flatten(),
    );
    let axis = 1;

    assert_eq!(
        Tensor::gather(&data, &indices, axis).unwrap(),
        Tensor::from_iter_f32(
            TensorType::new(ElementType::F32, vec![3, 1, 2]),
            [[[1.0, 1.9]], [[2.3, 3.9]], [[4.5, 5.9]],]
                .into_iter()
                .flatten()
                .flatten()
        )
    )
}
