use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid mask size: {0}")]
    InvalidMaskSize(String),
    #[error("Invalid cursor position: {0}")]
    InvalidCursorPosition(String),
}
