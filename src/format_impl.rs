use crate::float_impl::FormatFloat;
use crate::write_buffer::WriteBuffer;
use crate::Config;
use core::fmt::{Debug, Display, Write};

pub const BUFFER_SIZE: usize = 40;

pub(crate) trait FormatImpl: Copy {
    fn format_impl(self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize;
}

#[cfg(feature = "float32")]
impl FormatImpl for FormatFloat {
    fn format_impl(mut self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize {
        //dbg!(self, config);
        use crate::float_impl::*;
        #[allow(clippy::assertions_on_constants)]
        const _: () = {
            assert!(BUFFER_SIZE >= 30);
        };
        assert!(config.significant_digits <= 15);
        let is_negative = self.is_sign_negative();
        let out = if is_negative {
            out[0] = b'-';
            self = abs(self);
            &mut out[1..]
        } else {
            &mut out[..]
        };
        let writer = &mut WriteBuffer {
            buffer: out,
            written: 0,
        };
        if self.is_finite() {
            let mut config = Config { ..*config };
            if self != 0.0 {
                let log10 = MathImpl::floor(MathImpl::log10(self)) as i32;
                let target_log10 = config.significant_digits - 1;
                //dbg!(target_log10, log10);
                let float_shift = target_log10 as i32 - log10;
                self *= powi(10.0 as FormatFloat, float_shift);
                config.shift -= float_shift as isize;
                //dbg!(float_shift);
            }
            write!(writer, "{}", MathImpl::round(self) as u64).unwrap();
            //dbg!(String::from_utf8_lossy(&writer.buffer[..writer.written]));
            if cfg!(test) && self != 0.0 && writer.written != config.significant_digits {
                assert_eq!(writer.written, config.significant_digits + 1);
                assert!(
                    writer.buffer[0] == b'1'
                        && writer.buffer[1..writer.written].iter().all(|x| *x == b'0')
                );
            }
            let mut log10 = writer.written as isize + config.shift - 1;
            while writer.written < config.significant_digits {
                debug_assert!(self == 0.0);
                log10 = 0;
                writer.push_byte(b'0');
            }
            if writer.written != config.significant_digits {
                writer.written = config.significant_digits;
            }
            post_format_uint(writer, log10, config.significant_digits);
            writer.written + is_negative as usize
        } else {
            writer
                .write_str(if self.is_nan() {
                    "NaN"
                } else {
                    debug_assert!(self.is_infinite());
                    "inf"
                })
                .unwrap();
            writer.written + is_negative as usize
        }
    }
}

fn write_prefix(writer: &mut WriteBuffer, log1000: i32) {
    if !(-10..=10).contains(&log1000) {
        write!(writer, "e{log1000}").unwrap();
    } else if log1000 == -2 {
        writer.write_str("µ").unwrap();
    } else if log1000 != 0 {
        writer.push_byte(b"qryzafpnum kMGTPEZYRQ"[(log1000 + 10) as usize]);
    }
}

fn div_floor3(x: isize) -> isize {
    ((x.wrapping_sub(isize::MIN / 3 * 3) as usize) / 3) as isize + isize::MIN / 3
}

fn mod_floor3(x: isize) -> isize {
    x - div_floor3(x) * 3
}

macro_rules! impl_int {
    ($($signed:ty, $unsigned:ty,)*) => {
        $(
            impl FormatImpl for $unsigned {
            fn format_impl(self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize {
                format_unsigned(self, config, out)
            }
        }

        impl FormatImpl for $signed {
            fn format_impl(self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize {
                if self<0{
                    out[0]=b'-';
                    format_unsigned(self.wrapping_neg() as $unsigned,config,&mut out[1..])+1
                }else{
                    format_unsigned(self as $unsigned,config,out)
                }
            }
        }
        )*
    };
}

impl_int!(i32, u32, i64, u64, i128, u128,);

fn format_unsigned<T: Display + Debug>(x: T, config: &Config, buffer: &mut [u8]) -> usize {
    //dbg!(&x, config);
    let writer = &mut WriteBuffer { buffer, written: 0 };
    write!(writer, "{}", x).unwrap();
    //dbg!(String::from_utf8_lossy(writer.buffer));
    let mut digits = writer.written;
    while writer.written < config.significant_digits {
        writer.push_byte(b'0');
    }
    let round_up = writer.buffer[config.significant_digits] >= b'5';
    if round_up {
        let mut increment = config.significant_digits;
        loop {
            if increment == 0 {
                debug_assert!(writer.buffer[0..config.significant_digits]
                    .iter()
                    .all(|x| *x == b'0'));
                writer.buffer[0] = b'1';
                digits += 1;
                break;
            } else {
                increment -= 1;
                if writer.buffer[increment] == b'9' {
                    writer.buffer[increment] = b'0';
                } else {
                    writer.buffer[increment] += 1;
                    break;
                }
            }
        }
    }
    let log10 = digits as isize - 1 + config.shift;
    post_format_uint(writer, log10, config.significant_digits);
    writer.written
}

fn post_format_uint(writer: &mut WriteBuffer, log10: isize, significant_digits: usize) {
    let before_decimal = mod_floor3(log10) as usize + 1;
    // dbg!(String::from_utf8_lossy(&writer.buffer[..writer.written]),log10,significant_digits,before_decimal);
    if before_decimal < significant_digits {
        for digit in (before_decimal..significant_digits).rev() {
            let new_pos = digit + (digit - before_decimal) / 3 + 1;
            if digit > before_decimal && (digit - before_decimal) % 3 == 0 {
                writer.buffer[new_pos - 1] = b'_'
            }
            writer.buffer[new_pos] = writer.buffer[digit];
        }
        writer.buffer[before_decimal] = b'.';
        //dbg!(String::from_utf8_lossy(writer.buffer));
        let last_digit = significant_digits - 1;
        writer.written = last_digit + (last_digit - before_decimal) / 3 + 1 + 1;
    } else {
        writer.written = before_decimal;
    }
    write_prefix(writer, div_floor3(log10) as i32);
}
