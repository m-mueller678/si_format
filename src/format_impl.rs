use crate::float_impl::FormatFloat;
use crate::write_buffer::WriteBuffer;
use crate::Config;
use core::fmt::{Display, Write};

pub const BUFFER_SIZE: usize = 40;

pub(crate) trait FormatImpl: Copy {
    fn format_impl(self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize;
}

#[cfg(feature = "float32")]
impl FormatImpl for FormatFloat {
    fn format_impl(mut self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize {
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
        let mut writer = WriteBuffer {
            buffer: out,
            written: 0,
        };
        let std_precision = config.significant_digits - 1;
        if self.is_finite() {
            self *= powi(10 as FormatFloat, config.shift as i32);
            let log1000 = if self == 0.0 {
                0
            } else {
                MathImpl::floor(MathImpl::log10(self) / 3.0 - 1e-4) as i32
            };
            self *= powi(1000 as FormatFloat, -log1000);
            core::fmt::write(&mut writer, format_args!("{:.*}", std_precision, self)).unwrap();
            //dbg!(String::from_utf8_lossy(writer.buffer));
            let decimal_pos = writer.written - std_precision - 1;
            if writer.buffer[decimal_pos] == b'.' {
                if decimal_pos > 3 {
                    debug_assert!(decimal_pos == 4);
                    writer.buffer[1..][..4].rotate_right(1);
                }
                'format_number: {
                    if decimal_pos >= config.significant_digits {
                        writer.written = decimal_pos;
                        break 'format_number;
                    }
                    let decimal_places = config.significant_digits - decimal_pos;
                    for i in (0..decimal_places).rev() {
                        writer.buffer[decimal_pos + 1 + i + i / 3] =
                            writer.buffer[decimal_pos + 1 + i];
                        if i % 3 == 2 {
                            writer.buffer[decimal_pos + 1 + i + i / 3 + 1] = b'_';
                        }
                    }
                    let last_decimal = decimal_places - 1;
                    writer.written = decimal_pos + 1 + last_decimal + last_decimal / 3 + 1;
                    // at most 25
                };
            } else {
                debug_assert!(!writer.buffer[..writer.written].iter().any(|x| *x == b'.'));
            }
            write_prefix(&mut writer, log1000);
            debug_assert!(writer.written <= 29);
        } else {
            core::fmt::write(&mut writer, format_args!("{:.*}", std_precision, self)).unwrap();
        }
        writer.written + is_negative as usize
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

fn format_unsigned<T: Display>(x: T, config: &Config, buffer: &mut [u8]) -> usize {
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
    let before_decimal = mod_floor3(log10) as usize + 1;
    //dbg!(String::from_utf8_lossy(writer.buffer), );
    if before_decimal < config.significant_digits {
        for digit in (before_decimal..config.significant_digits).rev() {
            let new_pos = digit + (digit - before_decimal) / 3 + 1;
            if digit > before_decimal && (digit - before_decimal) % 3 == 0 {
                writer.buffer[new_pos - 1] = b'_'
            }
            writer.buffer[new_pos] = writer.buffer[digit];
        }
        writer.buffer[before_decimal] = b'.';
        //dbg!(String::from_utf8_lossy(writer.buffer));
        let last_digit = config.significant_digits - 1;
        writer.written = last_digit + (last_digit - before_decimal) / 3 + 1 + 1;
    } else {
        writer.written = before_decimal;
    }
    write_prefix(writer, div_floor3(log10) as i32);
    writer.written
}
