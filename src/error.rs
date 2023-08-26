use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Got an error when parsing config")]
    ParseConfig,
    #[error("Got an error: {0}")]
    #[allow(unused)]
    Other(&'static str),
}
