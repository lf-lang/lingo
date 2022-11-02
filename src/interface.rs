use crate::package::Package;

/// trait that all different build backends need to implement
pub trait Backend {
    fn from_package(package: Package) -> Self where Self: Sized;

    /// builds the package
    fn build(&self) -> bool;

    /// checks if the environment in nicely setup
    fn check(&self) -> bool;

    /// updates dependencies 
    fn update(&self) -> bool;

    /// build the package and runs it
    fn run(&self) -> bool;

    /// cleans the folder of any build arficacts
    fn clean(&self) -> bool;
}

