use std::path::{Path, PathBuf};
use std::{fs, io};

use which::which;

use crate::lfc::LFCProperties;
use crate::package::App;

pub mod analyzer;
pub mod command_line;

/// given is some list of build targets which are filtered by the binary regex
/// the lambda f is invoked on every element of the remaining elements which fit
/// the regex.
pub fn invoke_on_selected<F>(
    apps: &Vec<String>,
    sources: &Vec<App>,
    f: F,
) -> Result<(), Vec<io::Error>>
where
    F: Fn(&App) -> io::Result<()>,
{
    // evaluate f on every element inside sources and accumulate errors
    let errors: Vec<io::Error> = sources
        .iter()
        .filter(|&app| apps.is_empty() || apps.contains(&app.name))
        .map(f)
        .flat_map(|r| r.err())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

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

pub fn delete_subdirs(path_root: &mut PathBuf, subdirs: &[&str]) -> io::Result<()> {
    for &sub_dir in subdirs {
        path_root.push(sub_dir);
        if path_root.is_dir() {
            // ignore errors
            let _ = fs::remove_dir_all(&path_root);
        }
        path_root.pop();
    }

    Ok(())
}

pub fn default_build_clean(lfc: &LFCProperties) -> io::Result<()> {
    println!("removing build artifacts in {:?}", lfc.out);
    let mut path = lfc.out.clone();
    delete_subdirs(
        &mut path,
        &["bin", "include", "src-gen", "lib64", "share", "build"],
    )
}

pub fn find_lfc_exec(args: &crate::BuildArgs) -> Result<PathBuf, io::Error> {
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
