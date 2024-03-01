use std::path::Display;

pub mod args;
pub mod backends;
pub mod package;
pub mod util;
#[derive(Debug)]
pub enum WhichError {
    /// An executable binary with that name was not found
    CannotFindBinaryPath,
    /// There was nowhere to search and the provided name wasn't an absolute path
    CannotGetCurrentDirAndPathListEmpty,
    /// Failed to canonicalize the path found
    CannotCanonicalize,
}
#[derive(Debug)]
pub struct GitCloneError(String);

impl std::fmt::Display for WhichError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhichError::CannotFindBinaryPath => write!(f, "cannot find binary"),
            WhichError::CannotGetCurrentDirAndPathListEmpty => {
                write!(f, "cannot get current dir and path list empty")
            }
            WhichError::CannotCanonicalize => write!(f, "cannot canonicalize"),
        }
    }
}

impl std::fmt::Display for GitCloneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WhichError {}

impl std::error::Error for GitCloneError {}

pub type WhichType = fn(binary_name: &str) -> Result<std::path::PathBuf, WhichError>;
pub type GitCloneType = fn(url: &str, into: &std::path::Path) -> Result<(), GitCloneError>;
