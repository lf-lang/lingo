# Barrel

**Contact:** <tassilo-tanneberger@tu-dresden.de>

Barrel is a build tool for lingua-franca project it will manage dependencies 
configure build scripts and protentially cross compile for microcontrollers.

The Barrel.toml may look something like this.

```toml
[package]
name = "test"
version = "0.1.0"
language = "c"
main_reactor = [ "Main", "Test" ]

[dependencies]
lf-square = "0.1"
```

### command line interface

**Installing** is done on the most systems with `cargo install`.

```
lingua-franca package manager and build tool 0.1.0
tassilo.tanneberger@tu-dresden.de
Build system of lingua-franca projects

USAGE:
    barrel [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -b, --backend <BACKEND>    [default: cli]
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
