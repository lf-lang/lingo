use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// which backend to use
    #[clap(short, long, default_value_t = String::from("lfc"))]
    pub backend: String,

    /// which target to build
    #[clap(short, long)]
    pub target: Option<String>,

    /// overwrites any possible board definition in Lingo.toml
    #[clap(long)]
    pub board: Option<String>,

    /// tell lingo where the lfc toolchain can be found
    #[clap(short, long)]
    pub lfc: Option<String>,
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
    Run(BuildArgs),

    /// removes build artifacts
    Clean,
}

#[derive(Parser)]
#[clap(name = "lingua-franca package manager and build tool")]
#[clap(author = "tassilo.tanneberger@tu-dresden.de")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Build system of lingua-franca projects", long_about = None)]
pub struct CommandLineArgs {
    /// which command of lingo to use
    #[clap(subcommand)]
    pub command: Command,

    /// force lingo to use the specified backend
    #[clap(short, long, default_value_t = String::from("cli"))]
    pub backend: String,
}
