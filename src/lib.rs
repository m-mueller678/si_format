use core::fmt::{self, Display, Formatter};
use core::ops::ControlFlow;
use num_traits::PrimInt;
use std::fmt::Debug;

mod formattables;

pub trait Formattable {
    fn format_with(self, format: SiFormat) -> impl Display + Debug;
}

pub const SI_FORMAT: SiFormat = SiFormat {
    shift: 0,
    significant_digits: 3,
};

#[derive(Clone, Copy)]
pub struct SiFormat {
    shift: isize,
    significant_digits: usize,
}

impl SiFormat {
    pub fn with_precision(self, significant_digits: usize) -> Self {
        SiFormat {
            significant_digits,
            ..self
        }
    }

    pub fn with_shit(self, significant_digits: usize) -> Self {
        SiFormat {
            significant_digits,
            ..self
        }
    }

    pub fn f(self, x: impl Formattable) -> impl Display + Debug {
        x.format_with(self)
    }
}

struct SiFormatted<T: PrimInt> {
    num: T,
    format: SiFormat,
}

fn div_floor_3(x: isize) -> isize {
    (x + 900) / 3 - 300
}

trait Output<Inner> {
    type Error;
    fn write_byte(&mut self, i: &mut Inner, b: u8) -> Result<(), Self::Error>;
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

    #[inline]
    fn check_exponent(
        &mut self,
        i: &mut Formatter,
        e: isize,
    ) -> Result<ControlFlow<(), ()>, Self::Error> {
        if e < -30 {
            i.write_str("0")?;
            Ok(ControlFlow::Break(()))
        } else if e > 32 {
            i.write_str("∞")?;
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

impl<T: PrimInt> SiFormatted<T> {
    #[inline]
    fn output<I, O: Output<I>>(&self, out: &mut O, out_i: &mut I) -> Result<(), O::Error> {
        assert!(self.format.significant_digits <= 32);
        const DECIMAL_SEPARATOR: u8 = b'.';
        const GROUP_SEPARATOR: u8 = b'_';

        let mut buffer = [0u8; 32];
        let mut n = self.num;
        let mut buffer_i = 0usize;
        let mut digits_written = 0;
        while !n.is_zero() {
            digits_written += 1;
            buffer_i = if buffer_i == 0 {
                self.format.significant_digits
            } else {
                buffer_i
            } - 1;
            buffer[buffer_i] = (n % T::from(10).unwrap()).to_u8().unwrap();
            n = n / T::from(10).unwrap();
        }
        let msd = self.format.shift + digits_written as isize - 1;
        let msd3 = div_floor_3(msd);

        if out.check_exponent(out_i, msd)?.is_break() {
            return Ok(());
        }
        let mut pm3 = msd - msd3 * 3;
        let mut separator = DECIMAL_SEPARATOR;
        for i in (0..self.format.significant_digits).rev() {
            out.write_byte(out_i, buffer[buffer_i] + b'0')?;
            buffer_i += 1;
            if buffer_i == self.format.significant_digits {
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
    use crate::{SiFormat, SiFormatted};

    #[test]
    fn test() {
        fn t(num: usize, shift: isize, significant_digits: usize, expected: &str) {
            assert_eq!(
                SiFormatted {
                    num,
                    format: SiFormat {
                        shift,
                        significant_digits
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
