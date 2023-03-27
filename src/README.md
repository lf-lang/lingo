# Developer Documentation

## Architecture & Modules

General workflow that is happening.

```mermaid
graph TD
    A[Start] --> B(Handeling User Input)
    B --> C(Parsing Lingo.toml)
    C --> D(Configuring LFC)
    D --> E(Invoking LFC)
    E -->|--backend| F(Invoking Backend)
    F -->¦--no-compile¦ G(Done)
    G --> H(Done)
```

### Modules

- args
    command line arguments definition for lingo
- backend
    list of different backends which compile different targets
- lfc
    holding configuration structs for lingua-franca code generator
- util
    holding utility functions like capturing stdout of other programs
- package
    definition of the Lingo.toml schema
- interface
    defining the backend trait


