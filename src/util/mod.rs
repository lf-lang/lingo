use std::{fs, io};
use std::path::{Path, PathBuf};

use crate::package::App;

pub mod analyzer;
pub mod command_line;

/// given is some list of build targets which are filtered by the binary regex
/// the lambda f is invoked on every element of the remaining elements which fit
/// the regex.
pub fn invoke_on_selected<F>(apps: &Vec<String>, sources: &Vec<App>, f: F) -> Result<(), Vec<io::Error>>
    where
        F: Fn(&App) -> io::Result<()>,
{
    // evaluate f on every element inside sources and accumulate errors
    let errors: Vec<io::Error> =
        sources.iter()
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
    let path = match std::fs::canonicalize(input_path) {
        Ok(absolute_path) => absolute_path,
        Err(_) => {
            return None;
        }
    };

    match std::fs::read_dir(&path) {
        Ok(data) => {
            for element in data.flatten() {
                if element
                    .path()
                    .file_name()
                    .map_or_else(|| false, |file_name| file_name == "Lingo.toml")
                {
                    return Some(element.path());
                }
            }
            //return Some(path.to_path_buf());
        }
        Err(e) => {
            println!("cannot find toml file with error: {e:?}");
            return None;
        }
    };

    match path.parent() {
        Some(parent) => find_toml(parent),
        None => None,
    }
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
