use std::io;
use std::process::Command;

use crate::args::BuildArgs;
use crate::interface::Backend;
use crate::lfc::LFCProperties;
use crate::package::App;
use crate::util::command_line::run_and_capture;

pub struct LFC<'a> {
    target: &'a App,
    lfc: &'a LFCProperties,
}

impl<'a> Backend<'a> for LFC<'a> {
    fn from_target(target: &'a App, lfc: &'a LFCProperties) -> Self {
        LFC { target, lfc }
    }

    fn build(&self, _config: &BuildArgs) -> io::Result<()> {
        println!(
            "--- Building main reactor: {}",
            self.target.main_reactor.display()
        );
        // FIXME: What is this supposed to do? `lfc` does not have n `--output` argument
        //  also. Why isnt the lfc from the CLI `-l` used.
        let mut command = Command::new("lfc");
        command.arg("--output-path");
        command.arg("./");
        command.arg(&self.target.main_reactor);
        run_and_capture(&mut command).map(|_| ())
    }

    fn update(&self) -> bool {
        true
    }

    fn lfc(&self) -> &LFCProperties {
        self.lfc
    }
}
