pub mod command_line;

use crate::package::App;
use regex::Regex;
use std::path::PathBuf;
use std::fs::DirEntry;

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


pub fn find_root(path: &PathBuf) -> PathBuf {
    match std::fs::read_dir(path) {
        Ok(data) => {
            for path in data {
                println!("path: {:?}", &path);
            }
            return true;
        },
        Err(e) => {}
    };

    match path.parent() {
        Ok(parent) => find_root(parent),
        Err(e) => false
    }
}
