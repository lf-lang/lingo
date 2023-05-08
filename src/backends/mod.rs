pub mod lfc;

use crate::{args::BuildSystem, interface::Backend, package::App};

pub fn select_backend(name: &BuildSystem, app: &App) -> Option<Box<dyn Backend>> {
    match name {
        BuildSystem::LFC => {
            let lfc = lfc::LFC::from_target(app);
            Some(Box::new(lfc))
        }
        _ => {
            println!("error unkown backend!");
            None
        }
    }
}
