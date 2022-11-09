pub mod lfc;

use crate::{interface::Backend, package::Package};

pub fn select_backend(name: &str, package: &Package) -> Option<Box<dyn Backend>> {
    match name {
        "lfc" => {
            let lfc = lfc::LFC::from_package(package);
            Some(Box::new(lfc))
        }
        _ => {
            println!("error unkown backend!");
            None
        }
    }
}
