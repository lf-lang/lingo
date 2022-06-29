extern crate derive_builder;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    Init {

    },
    Build {

    },
    Generate {

    },
    Check {

    },
    Update {

    },
    Run {

    },
    Search  {
        package: String
    },
    Clean {

    },
    CollectGarbage {

    },
    Publish {

    },
}

#[derive(Parser, Debug)]
#[clap(name = "lingua-franca package manager")]
#[clap(author = "tassilo.tanneberger@tu-dresden.de")]
#[clap(version = "0.1.0")]
#[clap(about = "This program is a frontend for nix build system.", long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command
}
