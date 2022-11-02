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
    Publish {

    },
    Install {

    }
}

#[derive(Parser)]
#[clap(name = "lingua-franca package manager")]
#[clap(author = "tassilo.tanneberger@tu-dresden.de")]
#[clap(version = "0.1.0")]
#[clap(about = "Build system of lingua-franca projects", long_about = None)]
pub struct CommandLineArgs {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(short, long, default_value_t = String::from("cli"))]
    pub backend: String
}
