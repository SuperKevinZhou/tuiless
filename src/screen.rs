const FULL_SCROLLBACK: usize = usize::MAX;

pub struct ScreenBuffer {
    parser: vt100::Parser,
}

impl ScreenBuffer {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows.max(1), cols.max(1), FULL_SCROLLBACK),
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

    pub fn full_text(&mut self) -> String {
        let screen = self.parser.screen_mut();
        let original_scrollback = screen.scrollback();
        screen.set_scrollback(usize::MAX);
        let total_scrollback = screen.scrollback();
        let (_, cols) = screen.size();

        let mut rows = Vec::new();
        for offset in (0..=total_scrollback).rev() {
            screen.set_scrollback(offset);
            let visible_rows: Vec<String> = screen.rows(0, cols).collect();
            if rows.is_empty() {
                for (index, text) in visible_rows.into_iter().enumerate() {
                    rows.push((text, screen.row_wrapped(index as u16)));
                }
            } else if let Some((index, text)) = visible_rows.into_iter().enumerate().next_back() {
                rows.push((text, screen.row_wrapped(index as u16)));
            }
        }

        screen.set_scrollback(original_scrollback);
        render_rows(&rows)
    }
}

fn render_rows(rows: &[(String, bool)]) -> String {
    let mut output = String::new();
    let mut previous_wrapped = false;

    for (text, wrapped) in rows {
        if text.is_empty() && previous_wrapped {
            output.push('\n');
        }
        output.push_str(text);
        if !wrapped {
            output.push('\n');
        }
        previous_wrapped = *wrapped;
    }

    while output.ends_with('\n') {
        output.pop();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{ScreenBuffer, render_rows};

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

    #[test]
    fn full_text_includes_scrollback_history() {
        let mut screen = ScreenBuffer::new(8, 2);
        screen.apply(b"line1\r\nline2\r\nline3");
        assert_eq!(screen.viewport_text(), "line2\nline3");
        assert_eq!(screen.full_text(), "line1\nline2\nline3");
    }

    #[test]
    fn render_rows_preserves_wrapped_logical_lines() {
        let rows = vec![
            ("hello".to_string(), true),
            (" world".to_string(), false),
            ("tail".to_string(), false),
        ];
        assert_eq!(render_rows(&rows), "hello world\ntail");
    }
}
