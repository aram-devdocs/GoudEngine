use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error("This is a testing error for the goud_engine.")]
    TestError,
}