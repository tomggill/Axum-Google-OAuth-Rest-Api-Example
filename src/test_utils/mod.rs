#[macro_export]
macro_rules! assert_error {
    ($result_or_error:expr, $expected_type:expr) => {
        if !$result_or_error.is_err() {
            panic!("assert_err failed: expression is not an error")
        }

        let actual = $result_or_error.as_ref().unwrap_err();
        if std::mem::discriminant(actual) != std::mem::discriminant($expected_type) {
            panic!(
                "assert_err failed: error type does not match the expected type\nexpected: {:?}\nactual:   {:?}",
                $expected_type,
                actual
            )
        }
    }
}
