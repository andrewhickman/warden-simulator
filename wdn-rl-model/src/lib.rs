pub trait Action: Sized + Clone + Send + Sync {
    const SIZE: usize;

    fn as_u32(&self) -> u32;
    fn from_u32(value: u32) -> Self;

    fn argmax(weights: &[f32]) -> Self {
        debug_assert_eq!(Self::SIZE, weights.len());
        let (max, _) = weights
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap();
        Self::from_u32(max as u32)
    }
}

pub trait Observation: Sized + Clone + Send + Sync {
    const SIZE: usize;
    type ARRAY: AsRef<[f32]>;

    fn as_array(&self) -> Self::ARRAY;

    fn collect_into<T>(&self, collection: &mut T)
    where
        T: Extend<f32>,
    {
        collection.extend(self.as_array().as_ref().iter().copied())
    }
}

pub trait Model<O, A>: Send + Sync {
    fn react(&self, observation: &O) -> A;
}

impl<F, O, A> Model<O, A> for F
where
    F: Fn(&O) -> A + Send + Sync,
{
    fn react(&self, observation: &O) -> A {
        (self)(observation)
    }
}
