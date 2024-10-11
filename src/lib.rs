use crate::package::tree::GitLock;
use std::io;

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
pub struct GitCloneError(pub String); // TODO: create a more domain-specific error time like the actual git2::Error

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

pub struct GitUrl<'a>(&'a str);

impl<'a> From<&'a str> for GitUrl<'a> {
    fn from(value: &'a str) -> Self {
        GitUrl(value)
    }
}

impl<'a> From<GitUrl<'a>> for &'a str {
    fn from(value: GitUrl<'a>) -> Self {
        value.0
    }
}

pub type WhichCapability<'a> = Box<dyn Fn(&str) -> Result<std::path::PathBuf, WhichError> + 'a>;
pub type GitCloneCapability<'a> =
    Box<dyn Fn(GitUrl, &std::path::Path) -> Result<(), GitCloneError> + 'a>;
pub type FsReadCapability<'a> = Box<dyn Fn(&std::path::Path) -> io::Result<String> + 'a>;
pub type GitCloneAndCheckoutCap<'a> = Box<
    dyn Fn(GitUrl, &std::path::Path, Option<GitLock>) -> Result<Option<String>, GitCloneError> + 'a,
>;
