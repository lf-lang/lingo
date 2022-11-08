pub mod command_line;

use regex::Regex;

pub fn invoke_on_selected<F>(binary: Option<String>, mut sources: Vec<String>, f: F) -> bool
where
    F: Fn(&String) -> bool,
{
    // throws out all the sources that dont match the input regex
    binary.map(|filter| {
        // takes a binary contructs a regex out of it and checks
        // if a given source input matches the filter
        let regex_match = |input: &String| match Regex::new(&filter) {
            Ok(result) => result.is_match(input),
            Err(_) => false,
        };

        sources.retain(regex_match);
    });

    // evaluate f on everyelement inside sources and then compute the logical conjuction
    sources
        .iter()
        .map(f)
        .collect::<Vec<bool>>()
        .iter()
        .fold(true, |x, y| x && *y)
}