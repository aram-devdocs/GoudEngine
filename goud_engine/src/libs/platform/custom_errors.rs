use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]

pub enum Errors {
    #[error("This is a testing error for the goud_engine.")]
    TestError,
}
