#[macro_export]
macro_rules! round {
    ($number:expr, $places:expr) => {
        let scalar = 10.0_f64.powf(p as f64);
        (self * scalar).round() / scalar

        assert_almost_eq!($left, $right, $n, )
    };
}