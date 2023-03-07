# Developer Documentation


## Architecture & Modules

```mermaid
graph TD
    A[Start] --> B(Handeling User Input)
    B --> C(Parsing Lingo.toml)
    C --> D(Generating LFC-Struct)
    D --> E(Invoking LFC)
    E -->|--backend| F(Invoking Backend)
    E -->¦--no-compile¦ G(Done)
    F --> G(Done)
```

- analyzer
- backends
- lfc 
- package 
- util


## 







