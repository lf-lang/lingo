use crate::args::BuildArgs;
use crate::interface::Backend;
use crate::lfc::LFCProperties;
use crate::package::App;

use crate::util::command_line::run_and_capture;
use std::{env, io};
use std::fs;
use std::process::Command;

pub struct LFC<'a> {
    target: &'a App,
}

impl<'a> Backend<'a> for LFC<'a> {
    fn from_target(target: &'a App, _lfc: &'a LFCProperties) -> Self {
        LFC { target }
    }

    fn build(&self, _config: &BuildArgs) -> io::Result<()> {

        println!("building main reactor: {}", self.target.main_reactor);
        // FIXME: What is this supposed to do? `lfc` does not have n `--output` argument
        //  also. Why isnt the lfc from the CLI `-l` used.
        let mut command = Command::new("lfc");
        command.arg("--output");
        command.arg("./");
        command.arg(format!(
            "{}/{}",
            &self.target.root_path.display(),
            &self.target.main_reactor
        ));
        run_and_capture(&mut command).map(|_| ())
    }

    fn update(&self) -> bool {
        true
    }

    fn clean(&self) -> bool {
        println!("removing build artifacts in {:?}", env::current_dir());
        // just removes all the lingua-franca build artifacts
        fs::remove_dir_all("./bin").is_ok()
            && fs::remove_dir_all("./include").is_ok()
            && fs::remove_dir_all("./src-gen").is_ok()
            && fs::remove_dir_all("./lib64").is_ok()
            && fs::remove_dir_all("./share").is_ok()
            && fs::remove_dir_all("./build").is_ok()
    }
}
