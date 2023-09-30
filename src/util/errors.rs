use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use std::sync::Arc;

pub type AnyError = dyn Error + Send + Sync;
pub type BuildResult = Result<(), Box<AnyError>>;

#[derive(Debug)]
pub enum LingoError {
    Shared(Arc<AnyError>),
    CommandFailed(Command, ExitStatus),
    UnknownAppNames(Vec<String>),
    InvalidProjectLocation(PathBuf),
    UseWestBuildToBuildApp,
    InvalidMainReactor,
}

impl Display for LingoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LingoError::Shared(err) => {
                write!(f, "{}", err)
            }
            LingoError::CommandFailed(command, status) => {
                write!(f, "Command exited with status {}: {:?}", status, command)
            }
            LingoError::UnknownAppNames(names) => {
                write!(f, "Unknown app names: {}", names.join(", "))
            }
            LingoError::InvalidProjectLocation(path) => {
                write!(f, "Cannot initialize repository in {}", path.display())
            }
            LingoError::UseWestBuildToBuildApp => {
                write!(f, "Use `west lf-build` to build and run Zephyr programs.")
            }
            LingoError::InvalidMainReactor => {
                write!(
                    f,
                    "The main reactor was not a valid path to a file containing a main reactor"
                )
            }
        }
    }
}

impl Error for LingoError {}
