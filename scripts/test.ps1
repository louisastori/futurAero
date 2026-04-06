$ErrorActionPreference = "Stop"
cargo test --workspace
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
npm test
