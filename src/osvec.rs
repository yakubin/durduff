#[macro_export]
macro_rules! osvec {
    ( $( $x:expr ),* ) => {
        {
            vec![
                $(
                    OsString::from($x),
                )*
            ]
        }
    };
}
