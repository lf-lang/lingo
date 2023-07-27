use crate::backends::BuildProfile;
use clap::{Args, Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TargetLanguage {
    C,
    Cpp,
    Rust,
    TypeScript,
    Python,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Which build system to use
    /// TODO: discuss this
    #[clap(short, long)]
    pub build_system: Option<BuildSystem>,

    /// Which target to build
    #[clap(short, long)]
    pub language: Option<TargetLanguage>,

    /// Overwrites any possible board definition in Lingo.toml
    #[clap(long)]
    pub platform: Option<Platform>,

    /// Tell lingo where the lfc toolchain can be found
    #[clap(long)]
    pub lfc: Option<PathBuf>,

    /// Skips building aka invoking the build system so it only generates code
    #[clap(short, long, action)]
    pub no_compile: bool,

    /// If one of the apps fails to build dont interrupt the build process
    #[clap(short, long, action)]
    pub keep_going: bool,

    /// Compiles the binaries with optimizations turned on and strips debug symbols
    #[clap(short, long, action)]
    pub release: bool,

    /// List of apps to build if left empty all apps are built
    #[clap(short, long, value_delimiter = ',')]
    pub apps: Vec<String>,

    /// Number of threads to use for parallel builds. Zero means it will be determined automatically.
    #[clap(short, long, default_value_t = 0)]
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

impl ToString for TargetLanguage {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[clap(value_enum, short, long)]
    pub language: Option<TargetLanguage>,

    #[clap(value_enum, short, long)]
    pub platform: Option<Platform>,
}
impl InitArgs {
    pub fn get_target_language(&self) -> TargetLanguage {
        self.language.unwrap_or({
            // Target language for Zephyr and RP2040 is C 
            // Else use Cpp.
            match self.platform {
                Some(Platform::Zephyr) => TargetLanguage::C,
                Some(Platform::RP2040) => TargetLanguage::C,
                _ => TargetLanguage::Cpp,
            }
        })
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// initializing a lingua-franca project
    Init(InitArgs),

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

    /// lingo wouldn't produce any output
    #[clap(short, long, action)]
    pub quiet: bool,

    /// lingo wouldn't produce any output
    #[clap(short, long, action)]
    pub verbose: bool,
}
