use crate::{args::BuildArgs, package::App};

/// trait that all different build backends need to implement
pub trait Backend {
    fn from_target(target: &App) -> Self
    where
        Self: Sized;

    /// builds the package
    fn build(&self, config: &BuildArgs) -> bool;

    /// updates dependencies
    fn update(&self) -> bool;

    /// cleans the folder of any build arficacts
    fn clean(&self) -> bool;
}
