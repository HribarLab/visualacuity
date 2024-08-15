#[macro_export]
macro_rules! assert_almost_eq {
    ($left:expr, $right:expr, $n:expr) => { assert_almost_eq!($left, $right, $n, ) };
    ($left:expr, $right:expr, $n:expr, $($arg:tt)*) => {
        #[allow(unused_imports)]
        use crate::helpers::*;
        assert_eq!($left.round_places($n), $right.round_places($n), $($arg)*)
    };
}
