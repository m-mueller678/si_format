set -e

cargo clippy -- -D warnings
cargo clippy --no-default-features --features libm -- -D warnings
cargo clippy --no-default-features --features std -- -D warnings