use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Errors {
    #[error("This is a testing error for the goud_engine.")]
    TestError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_error() {
        let result = Errors::TestError;
        assert_eq!(result, Errors::TestError);
    }
}
