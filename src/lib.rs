#![warn(missing_docs)]

//! This crate formats numbers using metric prefixes:
//! ```
//! # use si_format::Formattable;
//! assert_eq!(123456u32.si_format().with_precision(3).to_string(),"123k")
//! ```
//! You may specify a shift by a certain number of decimal places.
//! This allows printing fractional quantities without floating point numbers:
//! ```
//! # use std::time::Duration;
//! # use si_format::Formattable;
//! let d = Duration::from_micros(20);
//! assert_eq!(format!("{}s",d.as_nanos().si_format().with_shift(-9)),"20.0µs");
//! ```

use core::fmt::{self, Display, Formatter};
use core::ops::ControlFlow;
use num_traits::PrimInt;
use std::fmt::Debug;

mod formattables;
mod format_impl;

/// A Type that can be formatted with a `SiFormat`.
pub trait Formattable {
    type Formatted: Display+Debug;
    /// Wraps self for formatting.
    /// The returned object can be further configured before display.
    fn si_format(self) -> Self::Formatted;
}

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
pub struct SiFormatted<T: PrimInt> {
    config: Config,
    num: T,
}

fn aaa(){
    0f64.fmt().unwrap()
}

impl<T: PrimInt> SiFormatted<T> {
    /// The number of significant digits to display, must be at least 3.
    /// ```
    /// use si_format::Formattable;
    /// assert_eq!(1234.si_format().with_precision(2).to_string(),"1.2k");
    /// ```
    pub const fn with_precision(mut self, significant_digits: usize) -> Self {
        assert!(significant_digits>=3);
        self.config.significant_digits = significant_digits;
        self
    }

    /// Multiply formatted value by a power of ten.
    ///
    /// The input number `x` is formatted as if it were `x*10^shift`.
    /// This allows formatting of fractional quantities using integers:
    /// ```
    /// use si_format::Formattable;
    /// assert_eq!(format!("{}s",(22).si_format().with_shift(-3)),"22.0ms");
    /// ```
    /// No actual multiplication is performed, the multiplied value need not be representable as `T`.
    pub const fn with_shift(mut self, shift: i8) -> Self {
        if self.config.shift!=isize::MAX{
            self.config.shift = shift as isize;
        }
        self
    }
}

fn div_floor_3(x: isize) -> isize {
    (x + 900) / 3 - 300
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

struct FormatOutput;

impl<T: PrimInt> Display for SiFormatted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.output(&mut FormatOutput, f)
    }
}

impl<T: PrimInt> Debug for SiFormatted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<'a> Output<Formatter<'a>> for FormatOutput {
    type Error = fmt::Error;

    #[inline]
    fn write_byte(&mut self, i: &mut Formatter, b: u8) -> Result<(), Self::Error> {
        i.write_str(core::str::from_utf8(&[b]).map_err(|_| ()).unwrap())
    }

    fn write_err(&mut self, i: &mut Formatter<'a>) -> Result<(), Self::Error> {
        i.write_str("ERR")
    }

    #[inline]
    fn check_exponent(
        &mut self,
        i: &mut Formatter,
        e: isize,
    ) -> Result<ControlFlow<(), ()>, Self::Error> {
        if e < -30 || e > 32 {
            self.write_err(i)?;
            Ok(ControlFlow::Break(()))
        } else {
            Ok(ControlFlow::Continue(()))
        }
    }

    #[inline]
    fn write_exponent(&mut self, i: &mut Formatter, e3: isize) -> Result<(), Self::Error> {
        if e3 == 0 {
            return Ok(());
        };
        if e3 == -2 {
            return i.write_str("µ");
        }
        let index = e3 + 10;
        self.write_byte(i, b"qryzafpnum kMGTPEZYRQ"[index as usize])
    }
}

// TODO fix rounding

impl<T: PrimInt> SiFormatted<T> {
    #[inline]
    fn output<I, O: Output<I>>(&self, out: &mut O, out_i: &mut I) -> Result<(), O::Error> {
        assert!(self.config.significant_digits <= 32);
        const DECIMAL_SEPARATOR: u8 = b'.';
        const GROUP_SEPARATOR: u8 = b'_';
        let mut buffer = [0u8; 32];
        let mut n = self.num;
        let mut buffer_i = 0usize;
        let mut digits_written = 0;
        while !n.is_zero() {
            digits_written += 1;
            buffer_i = if buffer_i == 0 {
                self.config.significant_digits
            } else {
                buffer_i
            } - 1;
            buffer[buffer_i] = (n % T::from(10).unwrap()).to_u8().unwrap();
            n = n / T::from(10).unwrap();
        }
        let msd = self.config.shift + digits_written as isize - 1;
        let msd3 = div_floor_3(msd);

        if out.check_exponent(out_i, msd)?.is_break() {
            return Ok(());
        }
        if self.num < T::from(0).unwrap() {
            out.write_byte(out_i, b'-')?;
        }
        let mut pm3 = msd - msd3 * 3;
        let mut separator = DECIMAL_SEPARATOR;
        for i in (0..self.config.significant_digits).rev() {
            out.write_byte(out_i, buffer[buffer_i] + b'0')?;
            buffer_i += 1;
            if buffer_i == self.config.significant_digits {
                buffer_i = 0
            };
            if pm3 == 0 && i != 0 {
                out.write_byte(out_i, separator)?;
                separator = GROUP_SEPARATOR;
                pm3 = 2
            } else {
                pm3 -= 1;
            }
        }
        out.write_exponent(out_i, msd3)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Config, SiFormatted};

    #[test]
    fn test() {
        fn t(num: usize, shift: isize, significant_digits: usize, expected: &str) {
            assert_eq!(
                SiFormatted {
                    num,
                    config: Config {
                        shift,
                        significant_digits,
                    },
                }
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
