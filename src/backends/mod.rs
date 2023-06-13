pub mod cmake;
pub mod lfc;
// pub mod cargo;

use std::io;
use crate::{args::BuildSystem, interface::Backend, lfc::LFCProperties, package::App};
use crate::args::BuildArgs;

pub fn run_build(name: BuildSystem, app: &App, lfc: &LFCProperties, args: &BuildArgs) -> io::Result<()> {
    match name {
        BuildSystem::LFC => lfc::LFC::do_build(app, lfc, args),
        BuildSystem::CMake => cmake::Cmake::do_build(app, lfc, args),
        //   BuildSystem::Cargo => cargo::CargoBackend::do_build(app, lfc, args)
    }
}
