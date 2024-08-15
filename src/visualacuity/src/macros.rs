#[macro_export]
macro_rules! round {
    ($number:expr, $places:expr) => {
        let scalar = 10.0_f64.powf(p as f64);
        (self * scalar).round() / scalar

        assert_almost_eq!($left, $right, $n, )
    };
}

#[macro_export]
macro_rules! impl_into_iter {
    ($type:ty, $inner:ty) => {
        impl IntoIterator for $type {
            type Item = <$inner as IntoIterator>::Item;
            type IntoIter = <$inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                return self.0.into_iter()
            }
        }
    };
}
