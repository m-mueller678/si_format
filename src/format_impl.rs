use crate::Config;
use std::fmt::Write;

pub const BUFFER_SIZE: usize = 32;

pub(crate) trait FormatImpl: Copy {
    fn format_impl(self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize;
}

impl FormatImpl for f64 {
    fn format_impl(mut self, config: &Config, out: &mut [u8; BUFFER_SIZE]) -> usize {
        assert!(self.is_finite() && self > 0.0);
        #[allow(clippy::assertions_on_constants)]
        const _: () = {
            assert!(BUFFER_SIZE >= 30);
        };
        assert!(config.significant_digits <= 15);
        self *= 10f64.powi(config.shift as i32);
        let log1000 = (self.log10() / 3.0 - 1e-4).floor() as i32;
        let normalized = self / 1000f64.powi(log1000);
        let mut writer = WriteBuffer {
            buffer: out,
            written: 0,
        };
        let std_precision = config.significant_digits - 1;
        core::fmt::write(
            &mut writer,
            format_args!("{:.*}", std_precision, normalized),
        )
        .unwrap();
        let decimal_pos = writer.written - std_precision - 1;
        //dbg!(String::from_utf8_lossy(&writer.buffer));
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
                    writer.buffer[decimal_pos + 1 + i + i / 3] = writer.buffer[decimal_pos + 1 + i];
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
        writer.written
    }
}

struct WriteBuffer<'a> {
    buffer: &'a mut [u8],
    written: usize,
}

impl core::fmt::Write for WriteBuffer<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let s = s.as_bytes();
        self.buffer[self.written..][..s.len()].copy_from_slice(s);
        self.written += s.len();
        Ok(())
    }
}

impl WriteBuffer<'_> {
    fn push_byte(&mut self, b: u8) {
        self.buffer[self.written] = b;
        self.written += 1;
    }
}
