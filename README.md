# Lingo - A build tool for Lingua Franca programs
**Contact:** <tassilo-tanneberger@tu-dresden.de>

Lingo is a one-stop build tool for the Lingua Franca project. 
Lingo will manage your dependencies, configure build scripts and will potentially cross-compile for embedded platforms.


## Getting started
Lingo is a Rust project and is built with cargo. To install it simply run
`cargo install --path .`

## The command line interface

```
Build system for the Lingua Franca coordination language

Usage: lingo [OPTIONS] <COMMAND>

Commands:
  init    initializing a lingua-franca project
  build   compiling one or multiple binaries in a lingua-franca package
  update  Updates the dependencies and potentially build tools
  run     builds and runs binaries
  clean   removes build artifacts
  help    Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet    lingo wouldn't produce any output
  -v, --verbose  lingo will give more detailed feedback
  -h, --help     Print help
  -V, --version  Print version
```

## The toml-based package configurations
The Lingo.toml may look something like this.

```toml
[package]
name = "example_project"
version = "0.1.0"
authors = ["tassilo.tannerber@tu-dresden.de"]
homepage = "https://lf-lang.org"
license = "Weird Stallman License"
description = "A little Lingo.toml for people"

# a library exported by this LF Package
[lib]
name = "websocket"
location = "./src/lib"
target = "C"
platform = "Native"

[lib.properties]
cmake-include="./websocket.cmake"

# first binary in the project
[[app]]
name = "git-hook"
target = "cpp"
main = "src/Main.lf"

# replacement for target properties
[[app.properties]]
cmake-include = "./my-cmake.cmake"
logging = "info"

# dependencies
[[dependencies]]
mqtt = {version=">=0.1", git="https://github.com/LF-Community/mqtt.git", branch="main"}

```

## Supported Platforms

We mainly support Linux and MacOs, support for windows is secondary.
