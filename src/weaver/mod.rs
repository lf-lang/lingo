
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::fs::{read_to_string, write};

#[derive(Deserialize, Serialize)]
pub struct Config {
    package: Package,
    dependencies: HashMap<String, String>

}

#[derive(Deserialize, Serialize)]
pub struct Package  {
    name: String,
    version: String,
    language: String,
    main_reactor: String,
    authors: Option<Vec<String>>,
    website: Option<String>,
    license: Option<String>,
    description: Option<String>,
    homepage: Option<String>
}


impl Config {
    pub fn new() -> Config {
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
                main_reactor: String::from("Main"),
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
        write(path, &toml_string);
    } 

    pub fn from(path: &Path) -> Config {
        let content = read_to_string(path).expect("Cannot read config file");
        toml::from_str(&content).expect("Cannot parse config")
    }

    pub fn write_nix_code(&self) {
        let derivation_code = self.to_nix();
        let flake_code: &'static str = include_str!("../../nix/flake.nix");

        std::fs::create_dir_all("./nix-build").expect("Cannot create target directory");

        write(Path::new("./nix-build/derivation.nix"), derivation_code);
        write(Path::new("./nix-build/flake.nix"), flake_code);
    }

    pub fn to_nix(&self) -> String {
        let mut meta_string = String::new();

        //TODO: check if language is one of cpp, c, rust, ts, python

        if self.package.description.is_some() {
            meta_string += &format!("description = \"{}\"", 
                                    self.package.description.as_ref().unwrap()).to_string(); 
        }

        if self.package.homepage.is_some() {
            meta_string += &format!("homepage = \"{}\"", 
                                    self.package.homepage.as_ref().unwrap()).to_string();
        }

        if self.package.license.is_some() {
            meta_string += &format!("license = \"{}\"", 
                                    self.package.license.as_ref().unwrap()).to_string();
        }

        if self.package.authors.is_some() {
            meta_string += &format!("maintainers = \"{:?}\"", 
                                    self.package.authors.as_ref().unwrap()).to_string();
        }

        let mut dependency_string = String::new();
        for (key, value) in &self.dependencies {
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

}


