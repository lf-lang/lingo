extern crate derive_builder;

use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// which binary to build
    #[clap(short, long)]
    pub package: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// initializing a lingua-franca project
    Init,

    /// compiling one ore multiple binaries in a lingua-franca package
    Build(BuildArgs),

    /// Updates the dependencies and potentially build tools
    Update,

    /// builds and runs binaries
    Run,

    /// removes build artifacts
    Clean,
}

#[derive(Parser)]
#[clap(name = "lingua-franca package manager and build tool")]
#[clap(author = "tassilo.tanneberger@tu-dresden.de")]
#[clap(version = "0.1.0")]
#[clap(about = "Build system of lingua-franca projects", long_about = None)]
pub struct CommandLineArgs {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(short, long, default_value_t = String::from("cli"))]
    pub backend: String,
}
