pub trait NewVec {
    fn new_vec<T: Default + Clone>(self) -> Vec<T>;
}

impl NewVec for usize {
    fn new_vec<T: Default + Clone>(self) -> Vec<T> {
        let mut result = Vec::with_capacity(self);
        result.resize(self, T::default());
        result
    }
}
