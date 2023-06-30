use crate::args::BuildArgs;
use crate::interface::Backend;
use crate::lfc::LFCProperties;
use crate::package::App;

use crate::util::command_line::run_and_capture;
use std::env;
use std::fs;
use std::process::Command;

pub struct LFC {
    target: App,
}

impl Backend for LFC {
    fn from_target(target: &App, _lfc: &LFCProperties) -> Self {
        LFC {
            target: target.clone(),
        }
    }

    fn build(&self, _config: &BuildArgs) -> bool {
        let reactor_copy = self.target.main_reactor.clone();

        let build_lambda = |main_reactor: &String| -> bool {
            println!("--- Building main reactor: {}", &main_reactor);
            // FIXME: What is this supposed to do? `lfc` does not have n `--output` argument
            //  also. Why isnt the lfc from the CLI `-l` used.
            let mut command = Command::new("lfc");
            command.arg("--output-path");
            command.arg("./");
            command.arg(format!(
                "{}/{}",
                &self.target.root_path.display(),
                &main_reactor
            ));
            run_and_capture(&mut command).is_ok()
        };

        if !build_lambda(&reactor_copy) {
            println!("--- LFC returned an error can lfc be found in $PATH ?");
            return false;
        }

        true
    }

    fn update(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        println!("--- Lingo removing build artifacts in {:?}", env::current_dir());
        // just removes all the lingua-franca build artifacts
        fs::remove_dir_all("./bin").is_ok()
            && fs::remove_dir_all("./include").is_ok()
            && fs::remove_dir_all("./src-gen").is_ok()
            && fs::remove_dir_all("./lib64").is_ok()
            && fs::remove_dir_all("./share").is_ok()
            && fs::remove_dir_all("./build").is_ok()
    }
}
