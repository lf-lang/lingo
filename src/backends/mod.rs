pub mod cmake;
pub mod lfc;
pub mod npm;

use crate::{args::BuildSystem, interface::Backend, lfc::LFCProperties, package::App};

pub fn select_backend(name: &BuildSystem, app: &App, lfc: &LFCProperties) -> Box<dyn Backend> {
    match name {
        BuildSystem::LFC => Box::new(lfc::LFC::from_target(app, lfc)),
        BuildSystem::CMake => Box::new(cmake::Cmake::from_target(app, lfc)),
        BuildSystem::Npm => Box::new(npm::Npm::from_target(app, lfc)),
    }
}
