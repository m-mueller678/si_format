set -ex

cargo clippy -- -D warnings
cargo clippy --no-default-features --features libm --features float32 -- -D warnings
cargo clippy --no-default-features --features std --features float32 -- -D warnings
cargo clippy --no-default-features --features libm --features float64 -- -D warnings
cargo clippy --no-default-features --features std --features float64 -- -D warnings


cargo test
cargo test --no-default-features --features libm --features float32
cargo test --no-default-features --features std --features float32
cargo test --no-default-features --features libm --features float64
cargo test --no-default-features --features std --features float64
