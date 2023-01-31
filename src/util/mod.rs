pub mod command_line;

use crate::package::App;
use regex::Regex;
use std::path::{PathBuf, Path};

/// given is some list of build targets which are filtered by the binary regex
/// the lambda f is invoked on every element of the remaining elements which fit
/// the regex.
pub fn invoke_on_selected<F>(binary: Option<String>, mut sources: Vec<App>, f: F) -> bool
where
    F: Fn(&App) -> bool,
{
    // throws out all the sources that dont match the input regex
    if let Some(filter) = binary {
        // takes a binary contructs a regex out of it and checks
        // if a given source input matches the filter
        let regex_match = |input: &App| match Regex::new(&filter) {
            Ok(result) => result.is_match(&input.name),
            Err(_) => false,
        };

        sources.retain(regex_match);
    }

    // evaluate f on everyelement inside sources and then compute the logical conjuction
    sources
        .iter()
        .map(f)
        .collect::<Vec<bool>>()
        .iter()
        .all(|y| *y)
}

/// finds toml file recurisvely
pub fn find_toml(input_path: &Path) -> Option<PathBuf> {
    let path = match std::fs::canonicalize(input_path) {
        Ok(absolute_path) => absolute_path,
        Err(_) => {return None;}
    };

    match std::fs::read_dir(&path) {
        Ok(data) => {
            for element in data {
                match element {
                    Ok(path_data) => {
                        if path_data.path().file_name().map_or_else(|| {false}, |file_name| {file_name == "Barrel.toml"})  {
                            return Some(path_data.path());
                        }
                    }
                    Err(_) => {}
                }

            }
            //return Some(path.to_path_buf());
        },
        Err(e) => {
            println!("cannot find toml file with error: {:?}", e);
            return None;
        }
    };

    match path.parent() {
        Some(parent) => find_toml(parent),
        None => None
    }
}


pub fn strip_file_name(path: &mut PathBuf) -> PathBuf {
    path.pop();
    println!("{:?}", &path);
    path.to_path_buf()
}
