use std::io;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use lazy_static::lazy_static;
use regex::Regex;

use crate::args::TargetLanguage;

lazy_static! {
    static ref TARGET_RE: Regex = Regex::new(r"\btarget\s+(\w+)\s*[{;]").unwrap();
}
lazy_static! {
    static ref MAIN_REACTOR_RE: Regex = Regex::new(r"\bmain\s+reactor\s+(\w+)\s*[{(]").unwrap();
}

const DEFAULT_TARGET: TargetLanguage = TargetLanguage::C;

/// this functions searches inside the file for a main reactor declaration
fn search_inside_file(path: &Path) -> io::Result<Option<MainReactorSpec>> {
    log::info!("Searching File {:?}", path);
    let content = std::fs::read_to_string(path)?;

    let mut target: TargetLanguage = DEFAULT_TARGET;
    for line in content.split('\n') {
        if let Some(captures) = MAIN_REACTOR_RE.captures(line) {
            target = TargetLanguage::from_str(captures.get(1).unwrap().as_str(), true).unwrap();
        }
        if let Some(captures) = TARGET_RE.captures(line) {
            let name = captures.get(1).unwrap().as_str().into();
            return Ok(Some(MainReactorSpec {
                name,
                target,
                path: path.to_path_buf(),
            }));
        };
    }
    Ok(None)
}

pub struct MainReactorSpec {
    pub name: String,
    pub target: TargetLanguage,
    pub path: PathBuf,
}

/// Searches for main reactors in the directory and descendants.
/// Return the simple name of main reactors.
/// TODO ideally use a language server service to find this out
pub fn find_main_reactors(path: &Path) -> io::Result<Vec<MainReactorSpec>> {
    fn acc_main_reactors(path: &mut PathBuf, result: &mut Vec<MainReactorSpec>) -> io::Result<()> {
        for entry in (std::fs::read_dir(&path)?).flatten() {
            path.push(entry.file_name());
            if path.is_dir() {
                acc_main_reactors(path, result)?;
            } else if path.is_file() {
                if let Some(main_reactor) = search_inside_file(path)? {
                    result.push(main_reactor);
                }
            }
            path.pop();
        }
        Ok(())
    }
    let mut main_reactors = Vec::new();
    acc_main_reactors(&mut path.to_path_buf(), &mut main_reactors)?;
    Ok(main_reactors)
}
