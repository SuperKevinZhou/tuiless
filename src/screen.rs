use std::cmp::min;

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cells: Vec<Vec<char>>,
    in_escape: bool,
    escape_buf: Vec<u8>,
}

impl ScreenBuffer {
    pub fn new(cols: u16, rows: u16) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        Self {
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            cells: vec![vec![' '; cols as usize]; rows as usize],
            in_escape: false,
            escape_buf: Vec::new(),
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols.max(1);
        self.rows = rows.max(1);
        self.cells.resize_with(self.rows as usize, || vec![' '; self.cols as usize]);
        for row in &mut self.cells {
            row.resize(self.cols as usize, ' ');
        }
        if self.cursor_y >= self.rows {
            self.cursor_y = self.rows - 1;
        }
        if self.cursor_x >= self.cols {
            self.cursor_x = self.cols - 1;
        }
    }

    pub fn apply(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.apply_byte(byte);
        }
    }

    pub fn viewport_text(&self) -> String {
        self.cells
            .iter()
            .map(|row| {
                let line: String = row.iter().collect();
                line.trim_end_matches(' ').to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn apply_byte(&mut self, byte: u8) {
        if self.in_escape {
            self.escape_buf.push(byte);
            if is_escape_terminated(byte) {
                self.handle_escape();
            }
            return;
        }

        match byte {
            0x1b => {
                self.in_escape = true;
                self.escape_buf.clear();
            }
            b'\r' => self.cursor_x = 0,
            b'\n' => self.newline(),
            0x08 => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            b'\t' => {
                let next_tab = ((self.cursor_x / 8) + 1) * 8;
                self.cursor_x = min(next_tab, self.cols - 1);
            }
            byte if byte.is_ascii_control() => {}
            byte => self.put_char(byte as char),
        }
    }

    fn handle_escape(&mut self) {
        let seq = String::from_utf8_lossy(&self.escape_buf).to_string();
        if let Some(rest) = seq.strip_prefix('[') {
            self.handle_csi(rest);
        }
        self.in_escape = false;
        self.escape_buf.clear();
    }

    fn handle_csi(&mut self, rest: &str) {
        if rest.is_empty() {
            return;
        }
        let final_char = rest.chars().last().unwrap();
        let body = &rest[..rest.len() - final_char.len_utf8()];
        let params: Vec<u16> = if body.is_empty() {
            Vec::new()
        } else {
            body.split(';')
                .filter_map(|part| {
                    if part.is_empty() || part == "?" {
                        None
                    } else {
                        part.trim_start_matches('?').parse::<u16>().ok()
                    }
                })
                .collect()
        };

        match final_char {
            'H' | 'f' => {
                let row = params.first().copied().unwrap_or(1).saturating_sub(1);
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.cursor_y = min(row, self.rows - 1);
                self.cursor_x = min(col, self.cols - 1);
            }
            'A' => {
                let amount = params.first().copied().unwrap_or(1);
                self.cursor_y = self.cursor_y.saturating_sub(amount);
            }
            'B' => {
                let amount = params.first().copied().unwrap_or(1);
                self.cursor_y = min(self.cursor_y.saturating_add(amount), self.rows - 1);
            }
            'C' => {
                let amount = params.first().copied().unwrap_or(1);
                self.cursor_x = min(self.cursor_x.saturating_add(amount), self.cols - 1);
            }
            'D' => {
                let amount = params.first().copied().unwrap_or(1);
                self.cursor_x = self.cursor_x.saturating_sub(amount);
            }
            'J' => {
                let mode = params.first().copied().unwrap_or(0);
                if mode == 2 || mode == 3 {
                    self.clear_all();
                }
            }
            'K' => {
                self.clear_line_from_cursor();
            }
            'm' => {}
            'h' | 'l' => {}
            't' => {}
            'r' => {}
            'S' | 'T' => {}
            _ => {}
        }
    }

    fn put_char(&mut self, ch: char) {
        if self.cursor_y >= self.rows {
            self.cursor_y = self.rows - 1;
        }
        if self.cursor_x >= self.cols {
            self.newline();
        }
        self.cells[self.cursor_y as usize][self.cursor_x as usize] = ch;
        if self.cursor_x + 1 >= self.cols {
            self.newline();
        } else {
            self.cursor_x += 1;
        }
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_y += 1;
        }
    }

    fn scroll_up(&mut self) {
        if !self.cells.is_empty() {
            self.cells.remove(0);
            self.cells.push(vec![' '; self.cols as usize]);
            self.cursor_y = self.rows - 1;
        }
    }

    fn clear_all(&mut self) {
        for row in &mut self.cells {
            row.fill(' ');
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn clear_line_from_cursor(&mut self) {
        if let Some(row) = self.cells.get_mut(self.cursor_y as usize) {
            for cell in row.iter_mut().skip(self.cursor_x as usize) {
                *cell = ' ';
            }
        }
    }
}

fn is_escape_terminated(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
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
        assert_eq!(screen.viewport_text(), "hell\no\n");
    }
}
