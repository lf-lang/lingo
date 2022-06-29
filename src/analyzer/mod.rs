use std::path::Path;

pub fn search_inside_file(path: &Path) -> Option<String> {
    println!("Searching File {:?}", path);
    let content = std::fs::read_to_string(path).expect("Cannot read config file");
    
    for line in content.split("\n") {
        if line.starts_with("main reactor") {
            return line.strip_prefix("main reactor ")
                .map(|x| 
                     x.to_string()
                     .split(" ")
                     .next()
                     .unwrap()
                     .to_string());
        };
    }
    None
}


pub fn search(path: &Path) -> Option<String> {
    for result_file in std::fs::read_dir(path).unwrap() {
        if result_file.is_err() {
            return None;
        }

        let file = result_file.unwrap().path();

        if file.is_dir() {
            let main_reactor = search(&file);
            if  main_reactor.is_some() {
                return main_reactor;
            }
        } else {
            let main_reactor = search_inside_file(&file);
            if  main_reactor.is_some() {
                return main_reactor;
            }
        }
    }
    None
}
