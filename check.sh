set -e

cargo fmt

FEATURES=(
    ""
    "--no-default-features"
    "--no-default-features --features libm --features float32"
    "--no-default-features --features std --features float32"
    "--no-default-features --features libm --features float64"
    "--no-default-features --features std --features float64"
)

for feature in "${FEATURES[@]}"; do
    echo $feature
    cargo clippy $feature -- -D warnings
done

for feature in "${FEATURES[@]}"; do
    echo $feature
    cargo test $feature
done
