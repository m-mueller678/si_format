#![warn(missing_docs)]

//! This crate formats numbers using metric prefixes:
//! ```
//! # use si_format::formattable::Formattable;
//! assert_eq!(123456u32.si_format().with_precision(3).to_string(),"123k")
//! ```
//! You may specify a shift by a certain number of decimal places.
//! This allows printing fractional quantities without floating point numbers:
//! ```
//! # use std::time::Duration;
//! # use si_format::formattable::Formattable;
//! let d = Duration::from_micros(20);
//! assert_eq!(format!("{}s",d.as_nanos().si_format().with_shift(-9)),"20.0µs");
//! ```

use crate::format_impl::{FormatImpl, BUFFER_SIZE};
use core::fmt::{self, Display, Formatter};
use core::ops::ControlFlow;
use std::fmt::Debug;

mod format_impl;
mod formattable;

struct Config {
    shift: isize,
    significant_digits: usize,
}

impl Config {
    /// A default configuration, the exact values are subject to change.
    pub const fn new() -> Self {
        Config {
            shift: 0,
            significant_digits: 3,
        }
    }
}

/// This is a number wrapped for formatting.
/// The `with_*` methods can be used to customize display.
pub struct SiFormatted<T> {
    config: Config,
    num: T,
}

impl<T: FormatImpl> Display for SiFormatted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buffer = [0u8; BUFFER_SIZE];
        let len = self.num.format_impl(&self.config, &mut buffer);
        f.write_str(std::str::from_utf8(&buffer[..len]).unwrap())
    }
}

impl<T> SiFormatted<T> {
    /// The number of significant digits to display, must be at least 3.
    /// ```
    /// use si_format::formattable::Formattable;
    /// assert_eq!(1234.si_format().with_precision(2).to_string(),"1.2k");
    /// ```
    pub const fn with_precision(mut self, significant_digits: usize) -> Self {
        assert!(significant_digits >= 3);
        self.config.significant_digits = significant_digits;
        self
    }

    /// Multiply formatted value by a power of ten.
    ///
    /// The input number `x` is formatted as if it were `x*10^shift`.
    /// This allows formatting of fractional quantities using integers:
    /// ```
    /// use si_format::formattable::Formattable;
    /// assert_eq!(format!("{}s",(22u64).si_format().with_shift(-3)),"22.0ms");
    /// ```
    /// No actual multiplication is performed, the multiplied value need not be representable as `T`.
    pub const fn with_shift(mut self, shift: i8) -> Self {
        if self.config.shift != isize::MAX {
            self.config.shift = shift as isize;
        }
        self
    }
}

trait Output<Inner> {
    type Error;
    fn write_byte(&mut self, i: &mut Inner, b: u8) -> Result<(), Self::Error>;
    fn write_err(&mut self, i: &mut Inner) -> Result<(), Self::Error>;
    fn check_exponent(
        &mut self,
        i: &mut Inner,
        e: isize,
    ) -> Result<ControlFlow<(), ()>, Self::Error>;
    fn write_exponent(&mut self, i: &mut Inner, e3: isize) -> Result<(), Self::Error>;
}

impl<T: FormatImpl> Debug for SiFormatted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::formattable::Formattable;

    #[test]
    fn test() {
        fn t<T: Formattable>(num: T, shift: i8, significant_digits: usize, expected: &str) {
            assert_eq!(
                num.si_format()
                    .with_shift(shift)
                    .with_precision(significant_digits)
                    .to_string(),
                expected
            );
        }

        t(12345678, -1, 3, "1.23M");
        t(12345678, -2, 3, "123k");
        t(12345678, -3, 3, "12.3k");
        t(12345678, -4, 3, "1.23k");
        t(12345678, -5, 3, "123");
        t(12345678, -5, 12, "123.456_780_000");
        t(12345678, -5, 8, "123.456_78");
        t(12345678, -5, 9, "123.456_780");
        t(123456789, -6, 9, "123.456_789");
    }
}
