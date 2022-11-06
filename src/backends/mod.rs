pub mod lfc;

use crate::{interface::Backend, package::Package};

pub fn select_backend(name: &str, package: Package) -> Box<dyn Backend> {
    match name {
        _ => {
            let lfc = lfc::LFC::from_package(package);
            Box::new(lfc)
        }
    }
}
