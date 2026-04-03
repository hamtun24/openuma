use thiserror::Error;

#[derive(Error, Debug)]
pub enum HwProbeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Command failed: {0}")]
    Command(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
