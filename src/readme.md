This crate formats numbers using metric prefixes:
```rust
assert_eq!(123456.si_format().to_string(),"123k")
```
# No-std
This crate fully supports no-std and environments without support for floating point numbers.

# Floating point
Formatting of floating point numbers is optionally available via the `float*` features.
These enable fromatting of floating point numbers up to the specific width (32 or 64 bit).
In addition, either `std` or `libm` need to be enabled for implementations of some required floating point functions.

