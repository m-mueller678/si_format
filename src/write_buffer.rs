pub(crate) struct WriteBuffer<'a> {
    pub buffer: &'a mut [u8],
    pub written: usize,
}

impl core::fmt::Write for WriteBuffer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        self.buffer[self.written..][..s.len()].copy_from_slice(s);
        self.written += s.len();
        Ok(())
    }
}

impl WriteBuffer<'_> {
    pub fn push_byte(&mut self, b: u8) {
        self.buffer[self.written] = b;
        self.written += 1;
    }
}
