use crate::Config;

const BUFFER_SIZE: usize = 24;

trait FormatImpl {
    fn format<'a>(self, config: Config, out: &mut [u8; BUFFER_SIZE])-> &'a [u8] ;
}

impl FormatImpl for f64 {
    fn format<'a>(mut self, config: Config, out: &'a mut [u8; BUFFER_SIZE]) -> &'a [u8] {
        debug_assert!(config.significant_digits <= 17);
        self *= 10f64.powi(config.shift as i32);
        let log1000 = (self.log10() / 3.0 - 1e-4).floor() as i32;
        let normalized = self / 1000f64.powi(log1000);
        let mut tmp_buffer = [0u8; 20];
        let mut tmp_buffer_write = WriteBuffer { buffer: &mut tmp_buffer, written: 0 };
        core::fmt::write(&mut tmp_buffer_write, format_args!("{:.*}", config.significant_digits - 1, normalized)).unwrap();
        let decimal_pos = tmp_buffer_write.written - (config.significant_digits - 1) - 1;
        debug_assert!(tmp_buffer[decimal_pos] == b'.');
        if decimal_pos > 3 {
            debug_assert!(decimal_pos == 4);
            tmp_buffer[1..][..4].rotate_right(1);
        }
        if decimal_pos >= config.significant_digits {
            return &tmp_buffer[..decimal_pos];
        }
        let decimal_places = config.significant_digits - decimal_pos - 1;
        for i in (0..decimal_places).rev() {
            tmp_buffer[decimal_pos + 1 + i + i / 3] = tmp_buffer[decimal_pos + 1 + i];
            if i % 3 == 2 {
                tmp_buffer[decimal_pos + 1 + i + i / 3 + 1] = b'_';
            }
        }
        let last_decimal = decimal_places - 1;
        &out[..=decimal_pos + 1 + last_decimal + last_decimal / 3]
    }
}

struct WriteBuffer<'a> {
    buffer: &'a mut [u8],
    written: usize,
}

impl core::fmt::Write for WriteBuffer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let s = s.as_bytes();
        self.buffer[self.written..][..s.len()].copy_from_slice(s);
        self.written += s.len();
        Ok(())
    }
}