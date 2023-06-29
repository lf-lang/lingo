use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

type AnyError = dyn Error + Send + Sync;
pub type BuildResult = Result<(), Box<AnyError>>;

#[derive(Debug)]
pub enum LingoError {
    Composite(Vec<Box<AnyError>>),
    CommandFailed(Command, ExitStatus),
    UnknownAppNames(Vec<String>),
    InvalidProjectLocation(PathBuf),
}

/// Merge two build results into one, collecting errors.
pub fn merge(b1: BuildResult, b2: BuildResult) -> BuildResult {
    match (b1, b2) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), e) | (e, Ok(())) => e,
        (Err(mut e1), Err(mut e2)) => {
            match (
                e1.downcast_mut::<LingoError>(),
                e2.downcast_mut::<LingoError>(),
            ) {
                (Some(LingoError::Composite(vec)), Some(LingoError::Composite(vec2))) => {
                    vec.append(vec2);
                    return Err(e1);
                }
                (Some(LingoError::Composite(vec)), _) => {
                    vec.push(e2);
                    Err(e1)
                }
                (_, Some(LingoError::Composite(vec))) => {
                    vec.push(e1);
                    Err(e2)
                }
                (_, _) => Err(Box::new(LingoError::Composite(vec![e1, e2]))),
            }
        }
    }
}

impl Display for LingoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LingoError::Composite(errors) => {
                for error in errors {
                    write!(f, "{}\n", error)?
                }
                Ok(())
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
        }
    }
}

impl Error for LingoError {}
