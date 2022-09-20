use super::search;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs::{read_to_string, write};
use termion::color;
use git2::Repository;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub package: Package,
    dependencies: HashMap<String, String>

}

#[derive(Deserialize, Serialize)]
pub struct Package  {
    pub name: String,
    version: String,
    pub language: String,
    main_reactor: String,
    authors: Option<Vec<String>>,
    website: Option<String>,
    license: Option<String>,
    description: Option<String>,
    homepage: Option<String>
}


impl Config {
    pub fn new() -> Config {
        let main_reactor;
        if !std::path::Path::new("./src").exists() {
            main_reactor = String::from("Main");
        } else {
            main_reactor = match search(Path::new("./src")) {
                Some(reactor) => reactor,
                _ => String::from("Main")
            };
        }

        Config {
            package: Package {
                name: std::env::current_dir()
                    .expect("error while reading current directory")
                    .as_path()
                    .file_name()
                    .expect("cannot get file name")
                    .to_string_lossy()
                    .to_string(),
                version: "0.1.0".to_string(),
                authors: None,
                language: "".to_string(),
                main_reactor: main_reactor,
                website: None,
                license: None,
                description: None,
                homepage: None
            },
            dependencies: HashMap::new()
        }
    }

    pub fn write(&self, path: &Path) {
        let toml_string = toml::to_string(&self).unwrap();
        write(path, &toml_string).expect("cannot write toml file");
    } 

    pub fn from(path: &Path) -> Config {
        let content = read_to_string(path).expect("Cannot read config file");
        toml::from_str(&content).expect("Cannot parse config")
    }

    pub fn setup_example() {
        if !std::path::Path::new("./src").exists() {
            std::fs::create_dir_all("./src").expect("Cannot create target directory");
            let hello_world_code: &'static str = include_str!("../../defaults/Main.lf");
            write(Path::new("./src/Main.lf"), hello_world_code).expect("cannot write Main.lf file!");
        }
    }

    pub fn write_nix_code(&self) {
        if ![ "cpp", "c", "rust", "ts", "python"].contains(&self.package.language.as_str()) {
                println!("{}Specified Language is not supported! Please specify the language in your Barrel.toml{}",
                color::Fg(color::Red), color::Fg(color::White));
            return;
        }

        let derivation_code = self.to_nix();
        let flake_code: &'static str = include_str!("../../defaults/flake.nix");

        std::fs::create_dir_all("./nix-build").expect("Cannot create target directory");

        write(Path::new("./nix-build/derivation.nix"), derivation_code).expect("Unable to write build files !");
        write(Path::new("./nix-build/flake.nix"), flake_code).expect("Unable to write build files !");
    }
    pub fn generate_meta_string(&self) -> String {
        let mut meta_string = String::new();
        //TODO: check if language is one of cpp, c, rust, ts, python

        if self.package.description.is_some() {
            meta_string += &format!("description = \"{}\";\n", 
                                    self.package.description.as_ref().unwrap()).to_string(); 
        }

        if self.package.homepage.is_some() {
            meta_string += &format!("homepage = \"{}\";\n", 
                                    self.package.homepage.as_ref().unwrap()).to_string();
        }

        if self.package.license.is_some() {
            meta_string += &format!("license = \"{}\";\n", 
                                    self.package.license.as_ref().unwrap()).to_string();
        }

        if self.package.authors.is_some() {
            meta_string += &format!("maintainers = {:?};\n", 
                                    self.package.authors.as_ref().unwrap()).to_string();
        }

        meta_string
    }
    pub fn to_nix(&self) -> String {
        let meta_string = self.generate_meta_string();

        let mut dependency_string = String::new();
        for (key, _value) in &self.dependencies {
            dependency_string += &key;
        }

        format!("
{{ pkgs, stdenv, lib, buildLinguaFranca, lfPackages}}: 
buildLinguaFranca {{
    name = \"{name}\";
    version = \"{version}\";
    src = ../.;
    language = \"{language}\";
    mainReactor = \"{mainreactor}\";
    buildInputs = with lfPackages; [ {dependencies} ];
    meta = with lib; {{
        {meta}
    }};
}}
            ", 
            dependencies = dependency_string, 
            name = self.package.name, 
            version = self.package.version,
            language = self.package.language, 
            mainreactor = self.package.main_reactor, 
            meta = meta_string
        )
    }

    pub fn publish_nix(&self) -> String {
        let meta_string = self.generate_meta_string();

        let mut dependency_string = String::new();
        for (key, _value) in &self.dependencies {
            dependency_string += &key;
        }

        let repo = match Repository::open("./") {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open git repo: {}", e),
        };

        let remote_name = match repo.find_remote("origin") {
            Ok(origin) => {
                origin.url().expect("Remote Url of origin is None").to_string().clone()
            }
            Err(_) => "".to_string()
        };
        let revision = match repo.head() {
            Ok(head) => format!("{}", head.peel_to_commit().unwrap().id()),
            Err(_) => "".to_string()
        };
        let comma = if dependency_string.len() == 0 {String::from("")} else {String::from(", ")};

        format!("
{{ pkgs, stdenv, lib, fetchgit, buildLinguaFranca{comma}{dependencies}}}: 
buildLinguaFranca {{
    name = \"{name}\";
    version = \"{version}\";
    src = fetchgit {{
        url = \"{remote_name}\";
        rev = \"{revision}\";
        sha256 = \"\";
    }};
    language = \"{language}\";
    mainReactor = \"{mainreactor}\";
    buildInputs = [ {dependencies} ];
    meta = with lib; {{
        {meta}
    }};
}}
            ", 
            comma = comma,
            remote_name = remote_name,
            revision = revision,
            dependencies = dependency_string, 
            name = self.package.name, 
            version = self.package.version,
            language = self.package.language, 
            mainreactor = self.package.main_reactor, 
            meta = meta_string
        )
    }
    pub fn root_nix_publish(&self) -> String {
        let mut dependency_string = String::new();
        for (key, _value) in &self.dependencies {
            dependency_string += &("  ".to_string() + key + " = " + key + ";\n");
        }

        format!("
{} = pkgs.callPackage ./{}/{}.nix {{
    buildLinguaFranca = buildLinguaFranca;
    {}
}}
            ", 
            self.package.name,
            self.package.language,
            self.package.name,
            dependency_string
        )
    }



}


