# Lingo - A build tool for Lingua Franca program
**Contact:** <tassilo-tanneberger@tu-dresden.de>

Lingo is a one-stop build tool for the Lingua Franca project. 
Lingo will manages dependencies, configure build scripts and will potentially cross-compile for embedded platforms.


## Getting started
Lingo is a Rust project and is built with cargo. To install it simply run
`cargo install --path .`

## The command line interfac

```
lingua-franca package manager and build tool 0.1.2
tassilo.tanneberger@tu-dresden.de
Build system of lingua-franca projects

USAGE:
    lingo [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -b, --backend <BACKEND>    force lingo to use the specified backend [default: cli]
    -h, --help                 Print help information
    -V, --version              Print version information

SUBCOMMANDS:
    build     compiling one ore multiple binaries in a lingua-franca package
    clean     removes build artifacts
    help      Print this message or the help of the given subcommand(s)
    init      initializing a lingua-franca project
    run       builds and runs binaries
    update    Updates the dependencies and potentially build tools
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

# shared properties of all binaries
[properties]
fast = true

# first binary in the project
[[app]]
name = "git-hook"
target = "cpp"
main_reactor = "src/Main.lf"
# main_reactor defaults to src/main.lf

# dependencies
[[app.dependencies]]
git = {version = "0.3.2"}
tarfetcher = {version = "0.4.2"}

# replacement for target properties
[[app.properties]]
cmake-include = "./my-cmake.cmake"
logging = true

# second binary
[[app]]
name = "embedded"
# main_reactor = "src/SayHello.lf"
target = "zephyr"

[[app.dependencies]]
blink = {version = "0.1.2"}

[[app.properties]]
no-compile = true
```
