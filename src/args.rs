use crate::backends::BuildProfile;
use clap::{Args, Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[value(rename_all = "lowercase")]
pub enum TargetLanguage {
    C,
    Cpp,
    Rust,
    TypeScript,
    Python,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum Platform {
    Native,
    Zephyr,
    RP2040,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum BuildSystem {
    LFC,
    CMake,
    Cargo,
    Npm,
    Pnpm,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Which build system to use
    /// TODO: discuss this
    #[arg(short, long)]
    pub build_system: Option<BuildSystem>,

    /// Which target to build
    #[arg(short, long)]
    pub language: Option<TargetLanguage>,

    /// Overwrites any possible board definition in Lingo.toml
    #[arg(long)]
    pub platform: Option<Platform>,

    /// Tell lingo where the lfc toolchain can be found
    #[arg(long)]
    pub lfc: Option<PathBuf>,

    /// Skips building aka invoking the build system so it only generates code
    #[arg(short, long)]
    pub no_compile: bool,

    /// If one of the apps fails to build dont interrupt the build process
    #[arg(short, long)]
    pub keep_going: bool,

    /// Compiles the binaries with optimizations turned on and strips debug symbols
    #[arg(short, long)]
    pub release: bool,

    /// List of apps to build if left empty all apps are built
    #[arg(short, long, value_delimiter = ',')]
    pub apps: Vec<String>,

    /// Number of threads to use for parallel builds. Zero means it will be determined automatically.
    #[arg(short, long, default_value_t = 0)]
    pub threads: usize,
}

impl BuildArgs {
    pub fn build_profile(&self) -> BuildProfile {
        if self.release {
            BuildProfile::Release
        } else {
            BuildProfile::Debug
        }
    }
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(value_enum, short, long)]
    pub language: Option<TargetLanguage>,
    #[arg(value_enum, short, long, default_value_t = Platform::Native)]
    pub platform: Platform,
}

impl InitArgs {
    pub fn get_target_language(&self) -> TargetLanguage {
        self.language.unwrap_or({
            // Target language for Zephyr and RP2040 is C
            // Else use Cpp.
            match self.platform {
                Platform::Zephyr => TargetLanguage::C,
                Platform::RP2040 => TargetLanguage::C,
                _ => TargetLanguage::Cpp,
            }
        })
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// initializing a lingua-franca project
    Init(InitArgs),

    /// compiling one or multiple binaries in a lingua-franca package
    Build(BuildArgs),

    /// Updates the dependencies and potentially build tools
    Update,

    /// builds and runs binaries
    Run(BuildArgs),

    /// removes build artifacts
    Clean,
}

#[derive(Parser)]
#[command(name = "Lingua Franca package manager and build tool")]
#[command(author = "tassilo.tanneberger@tu-dresden.de")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Build system for the Lingua Franca coordination language", long_about = None)]
pub struct CommandLineArgs {
    /// which command of lingo to use
    #[clap(subcommand)]
    pub command: Command,

    /// lingo wouldn't produce any output
    #[arg(short, long)]
    pub quiet: bool,

    /// lingo will give more detailed feedback
    #[arg(short, long)]
    pub verbose: bool,
}
