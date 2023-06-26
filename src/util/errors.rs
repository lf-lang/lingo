use std::error::Error;
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug)]
pub enum LingoError {
    CommandFailed(Command),
    RunAborted(),
}

impl Display for LingoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LingoError::RunAborted() => {
                write!(f, "Run aborted because --keep-going was not set")
            }
            LingoError::CommandFailed(command) => {
                write!(f, "Command failed {:?}", command)
            }
        }
    }
}

impl Error for LingoError {}



