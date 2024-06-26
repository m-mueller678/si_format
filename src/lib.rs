#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

//! This crate formats numbers using metric prefixes:
//! ```
//! # use si_format::Formattable;
//! assert_eq!(123456.si_format().to_string(),"123k")
//! ```
//! You may specify a shift by a certain number of decimal places.
//! This is particularly useful for integers that represent a fixed point fraction:
//! ```
//! # use core::time::Duration;
//! # use si_format::Formattable;
//! let d = Duration::from_micros(20);
//! assert_eq!(format!("{}s",d.as_nanos().si_format().with_shift(-9)),"20.0µs");
//! ```
//! # Rounding
//! Currently, the method used for rounding is unspecified. `0.5` may be rounded either up or down.
//!
//! # No-std
//! This crate fully supports no-std and environments without support for floating point numbers.
//!
//! # Floating point
//! Formatting of floating point numbers is optionally available via the `float*` features.
//! These enable fromatting of floating point numbers up to the specific width (32 or 64 bit).
//! In addition, either `std` or `libm` need to be enabled for implementations of some required floating point functions.
//!
extern crate alloc;
extern crate core;

use crate::format_impl::BUFFER_SIZE;
use core::fmt::Debug;
use core::fmt::{self, Display, Formatter};
use format_impl::FormatImpl;

#[cfg(feature = "float32")]
mod float_impl;
mod format_impl;
mod formattable;
mod write_buffer;

pub use formattable::Formattable;

#[derive(Debug)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut buffer = [0u8; BUFFER_SIZE];
        let len = self.num.format_impl(&self.config, &mut buffer);
        f.write_str(core::str::from_utf8(&buffer[..len]).unwrap())
    }
}

impl<T> SiFormatted<T> {
    /// The number of significant digits to display.
    /// Unlike the precision for `std::fmt`, this includes digits before the decimal point.
    /// ```
    /// use si_format::Formattable;
    /// assert_eq!(1234567.si_format().with_precision(5).to_string(),"1.234_6M");
    /// ```
    /// Currently, this must be at least 3.
    // At lower precision we might get rounding before the decimal separator: 123 -> 120.
    pub fn with_precision(mut self, significant_digits: usize) -> Self {
        assert!(significant_digits >= 3);
        self.config.significant_digits = significant_digits;
        self
    }

    /// Multiply formatted value by a power of ten.
    ///
    /// The input number `x` is formatted as if it were `x*10^shift`.
    /// This allows formatting of fractional quantities using integers:
    /// ```
    /// use si_format::Formattable;
    /// assert_eq!(format!("{}s",(22u64).si_format().with_shift(-3)),"22.0ms");
    /// ```
    /// No actual multiplication is performed, the multiplied value need not be representable as `T`.
    pub fn with_shift(mut self, shift: i8) -> Self {
        if self.config.shift != isize::MAX {
            self.config.shift = shift as isize;
        }
        self
    }
}

impl<T: FormatImpl> Debug for SiFormatted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "float32")]
    use crate::float_impl::FormatFloat;
    use crate::formattable::Formattable;
    use crate::write_buffer::WriteBuffer;
    use core::fmt::Display;
    use core::fmt::Write;
    use core::ops::Neg;

    #[test]
    fn test() {
        fn to_string(buffer: &mut [u8], x: impl Display) -> &str {
            let writer = &mut WriteBuffer { buffer, written: 0 };
            write!(writer, "{}", x).unwrap();
            let written = writer.written;
            core::str::from_utf8(&buffer[..written]).unwrap()
        }

        fn t<T: Formattable + Neg<Output = T> + Copy>(
            num: T,
            shift: i8,
            significant_digits: usize,
            expected: &str,
        ) {
            assert_eq!(
                to_string(
                    &mut [0u8; 300],
                    num.si_format()
                        .with_shift(shift)
                        .with_precision(significant_digits)
                ),
                expected
            );
            assert_eq!(
                to_string(
                    &mut [0u8; 300],
                    (-num)
                        .si_format()
                        .with_shift(shift)
                        .with_precision(significant_digits)
                ),
                to_string(&mut [0u8; 300], format_args!("-{}", expected))
            );
        }

        t(12345678, -1, 3, "1.23M");
        t(12345678, -2, 3, "123k");
        t(12345678, -3, 3, "12.3k");
        t(12345678, -4, 3, "1.23k");
        t(12345678, -5, 3, "123");
        #[cfg(feature = "float64")]
        {
            t(12345678, -5, 12, "123.456_780_000");
            t(12345678, -5, 8, "123.456_78");
            t(12345678, -5, 9, "123.456_780");
            t(123456789, -6, 9, "123.456_789");
            t(121212121212121212121212121i128, 0, 12, "121.212_121_212Y");
        }
        #[cfg(feature = "float64")]
        t(121212121212121212121212121f64, 0, 12, "121.212_121_212Y");
        t(9999, 0, 3, "10.0k");
        t(99999, 0, 4, "100.0k");
        t(999999, 0, 5, "1.000_0M");
        #[cfg(feature = "float32")]
        {
            t(1.3e-4, 0, 3, "130µ");
            t(1.3e-4, 0, 4, "130.0µ");
            t(0.0, 0, 4, "0.000");
            t(FormatFloat::INFINITY, 0, 4, "inf");
            t(FormatFloat::INFINITY, -8, 4, "inf");
            t(FormatFloat::NAN, 0, 4, "NaN");
            t(FormatFloat::NAN, 2, 4, "NaN");
        }
    }
}
