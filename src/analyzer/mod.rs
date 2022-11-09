use std::path::Path;

pub fn search_inside_file(path: &Path) -> Option<String> {
    println!("Searching File {:?}", path);
    let content = std::fs::read_to_string(path).expect("Cannot read config file");

    for line in content.split('\n') {
        if line.starts_with("main reactor") {
            return line
                .strip_prefix("main reactor ")
                .map(|x| x.to_string().split(' ').next().unwrap().to_string());
        };
    }
    None
}

pub fn search(path: &Path) -> Vec<String> {
    let mut main_reactors = Vec::new();

    for result_file in std::fs::read_dir(path).unwrap() {
        if result_file.is_err() {
            continue;
        }

        let file = result_file.unwrap().path();

        if file.is_dir() {
            let main_reactor = search(&file);
            main_reactors.append(&mut main_reactor.clone())
        } else {
            let optional_main_reactor = search_inside_file(&file);
            if let Some(main_reactor) = optional_main_reactor{
                main_reactors.push(main_reactor);
            }
        }
    }

    main_reactors
}
