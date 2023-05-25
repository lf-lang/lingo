use crate::args::BuildArgs;
use crate::interface::Backend;
use crate::lfc::LFCProperties;
use crate::App;

use crate::util::command_line::run_and_capture;
use std::env;
use std::fs;
use std::process::Command;
use which::which;

pub struct Npm {
    app: App,
    lfc: LFCProperties,
}

impl Backend for Npm {
    fn from_target(target: &App, lfc: &LFCProperties) -> Self {
        Npm {
            app: target.clone(),
            lfc: lfc.clone(),
        }
    }

    fn build(&self, config: &BuildArgs) -> bool {
        // check if pnpm is available
        let mut cmd = "npm";
        let mut prod: &str = "--production";
        if which("pnpm").is_ok() {
            cmd = "pnpm";
            prod = "--prod"
        } else if !which("npm").is_ok() {
            // error
            return false;
        }
        
        // install
        let mut npm_install = Command::new(cmd);
        npm_install.arg("install");
        if config.release {
            npm_install.arg(prod);
        }

        npm_install.current_dir(format!("{}", self.lfc.out.display().to_string()));
        let npm_installed = run_and_capture(&mut npm_install).is_ok();
        let runtime_built: bool = true;

        if cmd.eq("npm") {
            // FIXME: build reactor-ts when pulling from GitHub instead of NPM
            
        }
        npm_installed && runtime_built
    }

    fn update(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        println!("removing build artifacts in {:?}", env::current_dir());
        // just removes all the lingua-franca build artifacts
        fs::remove_dir_all("./node_modules").is_ok()
            && fs::remove_dir_all("./dist").is_ok()
    }
}
