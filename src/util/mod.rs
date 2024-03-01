use std::path::{Path, PathBuf};
use std::{fs, io};

use which::which;

pub mod analyzer;
mod command_line;
pub mod errors;

pub use command_line::*;

/// finds toml file recurisvely
pub fn find_toml(input_path: &Path) -> Option<PathBuf> {
    let mut path = fs::canonicalize(input_path).ok()?;
    while path.is_dir() {
        path.push("Lingo.toml");
        if path.is_file() {
            return Some(path);
        }
        path.pop(); // remove Lingo.toml
        if !path.pop() {
            // cannot pop more
            break;
        }
    }
    None
}

/// Copy files from source to destination recursively.
// Copied from https://nick.groenen.me/notes/recursively-copy-files-in-rust/
pub fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn delete_subdirs(path_root: &Path, subdirs: &[&str]) -> io::Result<()> {
    let mut buf = path_root.to_owned();
    for &sub_dir in subdirs {
        buf.push(sub_dir);
        if buf.is_dir() {
            // ignore errors
            let _ = fs::remove_dir_all(&buf);
        }
        buf.pop();
    }

    Ok(())
}

pub fn default_build_clean(out_dir: &Path) -> io::Result<()> {
    println!("removing build artifacts in {:?}", out_dir);
    delete_subdirs(
        out_dir,
        &["bin", "include", "src-gen", "lib64", "share", "build"],
    )
}

pub fn find_lfc_exec(args: &crate::args::BuildArgs) -> Result<PathBuf, io::Error> {
    if let Some(lfc) = &args.lfc {
        if lfc.exists() {
            return Ok(lfc.clone());
        }
    } else if let Ok(lfc) = which("lfc") {
        return Ok(lfc);
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "LFC executable not found",
    ))
}
