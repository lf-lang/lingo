use crate::package::Package;

/// trait that all different build backends need to implement
pub trait Backend {
    fn from_package(package: &Package) -> Self
    where
        Self: Sized;

    /// builds the package
    fn build(&self, package: Option<String>) -> bool;

    /// updates dependencies
    fn update(&self) -> bool;

    /// cleans the folder of any build arficacts
    fn clean(&self) -> bool;
}
