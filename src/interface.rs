use crate::lfc::LFCProperties;
use crate::{args::BuildArgs, package::App};
use std::io;

/// trait that all different build backends need to implement
pub trait Backend<'a> {
    fn do_build(app: &'a App, lfc: &'a LFCProperties, args: &BuildArgs) -> io::Result<()>
    where
        Self: Sized,
    {
        let me = Self::from_target(app, lfc);
        me.build(args)
    }

    fn from_target(target: &'a App, lfc: &'a LFCProperties) -> Self
    where
        Self: Sized;

    /// builds the package
    fn build(&self, config: &BuildArgs) -> io::Result<()>;

    /// updates dependencies
    fn update(&self) -> bool;

    fn lfc(&self) -> &LFCProperties;

    /// cleans the folder of any build arficacts
    fn clean(&self) -> io::Result<()> {
        crate::util::default_build_clean(self.lfc())
    }
}
