use crate::Config;
use core::fmt::Write;
use crate::float_impl::FormatFloat;

pub const BUFFER_SIZE: usize = 32;

pub(crate) trait FormatImpl: Copy {
    fn format_impl(self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize;
}

#[cfg(feature = "float32")]
impl FormatImpl for FormatFloat {
    fn format_impl(mut self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize {
        use crate::float_impl::*;
        use crate::write_buffer::WriteBuffer;
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
            if !(-10..=10).contains(&log1000) {
                writer.push_byte(b'e');
                write!(&mut writer, "{log1000}").unwrap();
            } else if log1000 == -2 {
                writer.write_str("µ").unwrap();
            } else if log1000 != 0 {
                writer.push_byte(b"qryzafpnum kMGTPEZYRQ"[(log1000 + 10) as usize]);
            }
            debug_assert!(writer.written <= 29);
        } else {
            core::fmt::write(&mut writer, format_args!("{:.*}", std_precision, self)).unwrap();
        }
        writer.written + is_negative as usize
    }
}
