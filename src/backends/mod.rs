pub mod lfc;

use crate::{interface::Backend, package::App};

pub fn select_backend(name: &str, app: &App) -> Option<Box<dyn Backend>> {
    match name {
        "lfc" => {
            let lfc = lfc::LFC::from_target(app);
            Some(Box::new(lfc))
        }
        _ => {
            println!("error unkown backend!");
            None
        }
    }
}
