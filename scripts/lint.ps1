$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --check
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets -- -D warnings
npm run lint
