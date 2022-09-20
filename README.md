# Barrel

**Contact:** <revol-xut@protonmail.com>

Package manager for the lingua-franca which uses lf-pkgs
functions as a backend. This tool transpiles the Barrel.toml 
into nix code which then is used to build the actuall package.

The Barrel.toml may look something like this.

```toml

```


### command line interface

```
lingua-franca package manager 0.1.0
tassilo.tanneberger@tu-dresden.de
This program is a frontend for nix build system.

USAGE:
    barrel <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    build
    check
    clean
    collect-garbage
    generate
    help               Print this message or the help of the given subcommand(s)
    init
    install
    publish
    run
    search
    update
```
