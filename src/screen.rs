pub struct ScreenBuffer {
    parser: vt100::Parser,
}

impl ScreenBuffer {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows.max(1), cols.max(1), 0),
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.parser.screen_mut().set_size(rows.max(1), cols.max(1));
    }

    pub fn apply(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn viewport_text(&self) -> String {
        self.parser.screen().contents()
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenBuffer;

    #[test]
    fn viewport_text_is_trimmed_and_visible_only() {
        let mut screen = ScreenBuffer::new(5, 2);
        screen.apply(b"ab\r\nxy");
        assert_eq!(screen.viewport_text(), "ab\nxy");
    }

    #[test]
    fn resize_keeps_viewport_dimensions() {
        let mut screen = ScreenBuffer::new(4, 2);
        screen.apply(b"hello");
        screen.resize(6, 3);
        assert_eq!(screen.viewport_text(), "hell\no");
    }
}
