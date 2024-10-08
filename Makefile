pkg: Cargo.toml Cargo.lock $(wildcard src/*)
	wasm-pack build . || npx -y wasm-pack build .
