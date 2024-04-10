This crate formats numbers using metric prefixes:
```rust
assert_eq!(123456.si_format().to_string(),"123k");
```

This provides more limited functionality than [si-scale](https://crates.io/crates/si-scale), but works without std, alloc, and (optionally) floating point arithmetic.
This is primarily intended to make numbers in logs and debug printing more readable at a minimum runtime cost.
