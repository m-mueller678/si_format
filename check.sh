set -e

cargo clippy -- -D warnings
cargo clippy --no-default-features --features libm -- -D warnings
cargo clippy --no-default-features --features std -- -D warnings

cargo test
cargo test --no-default-features --features libm
cargo test --no-default-features --features std
