use std::collections::{HashMap, HashSet};

use anyhow::Result;
use vt100::Color;

use crate::protocol::{SnapshotColorLayer, SnapshotColorMode, SnapshotTheme};

const FULL_SCROLLBACK: usize = usize::MAX;
const ANSI_SLOT_SYMBOLS: [char; 16] = [
    'k', 'r', 'g', 'y', 'b', 'p', 'c', 'w', 'K', 'R', 'G', 'Y', 'B', 'P', 'C', 'W',
];
const DYNAMIC_SYMBOL_SOURCE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const MAX_DYNAMIC_SYMBOLS: usize = 36;

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

    pub fn viewport_ansi_text(&self) -> String {
        let formatted = self.parser.screen().state_formatted();
        String::from_utf8_lossy(&formatted).into_owned()
    }

    pub fn viewport_color_text(
        &self,
        mode: SnapshotColorMode,
        theme: SnapshotTheme,
    ) -> Result<String> {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let mut codebook = ColorCodebook::new();
        let mut sections = Vec::new();

        sections.push(render_character_grid(screen, rows, cols));

        for layer in mode.ordered_layers() {
            let use_smart = mode == SnapshotColorMode::Smart;
            sections.push(render_color_grid(
                screen,
                rows,
                cols,
                *layer,
                use_smart,
                &mut codebook,
            )?);
        }

        let mut output = sections.join("\n\n");
        output.push_str("\n\nLegend:\n");
        output.push_str(&codebook.legend_lines(theme).join("\n"));
        Ok(output)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DynamicColorKey {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

struct ColorCodebook {
    used_ansi_slots: [bool; 16],
    dynamic_symbols: Vec<char>,
    dynamic_by_key: HashMap<DynamicColorKey, char>,
    dynamic_order: Vec<(char, DynamicColorKey)>,
}

impl ColorCodebook {
    fn new() -> Self {
        Self {
            used_ansi_slots: [false; 16],
            dynamic_symbols: dynamic_symbol_pool(),
            dynamic_by_key: HashMap::new(),
            dynamic_order: Vec::new(),
        }
    }

    fn symbol_for(&mut self, color: Color) -> Result<char> {
        match color {
            Color::Default => Ok('.'),
            Color::Idx(slot) if slot < 16 => {
                self.used_ansi_slots[usize::from(slot)] = true;
                Ok(ANSI_SLOT_SYMBOLS[usize::from(slot)])
            }
            Color::Idx(index) => self.assign_dynamic_symbol(DynamicColorKey::Indexed(index)),
            Color::Rgb(r, g, b) => self.assign_dynamic_symbol(DynamicColorKey::Rgb(r, g, b)),
        }
    }

    fn assign_dynamic_symbol(&mut self, key: DynamicColorKey) -> Result<char> {
        if let Some(symbol) = self.dynamic_by_key.get(&key).copied() {
            return Ok(symbol);
        }

        let next_symbol = self.dynamic_symbols.get(self.dynamic_order.len()).copied();
        let symbol = next_symbol.ok_or_else(|| {
            anyhow::anyhow!(
                "color export supports up to {MAX_DYNAMIC_SYMBOLS} dynamic colors per snapshot"
            )
        })?;
        self.dynamic_by_key.insert(key, symbol);
        self.dynamic_order.push((symbol, key));
        Ok(symbol)
    }

    fn legend_lines(&self, theme: SnapshotTheme) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(".=default".to_string());

        for slot in 0u8..16 {
            if self.used_ansi_slots[usize::from(slot)] {
                let symbol = ANSI_SLOT_SYMBOLS[usize::from(slot)];
                let hex = theme
                    .ansi16_hex(slot)
                    .unwrap_or("#000000")
                    .to_ascii_lowercase();
                lines.push(format!("{symbol}=ansi{slot}({hex})"));
            }
        }

        for (symbol, key) in &self.dynamic_order {
            match key {
                DynamicColorKey::Indexed(index) => {
                    let (r, g, b) = xterm_index_to_rgb(*index);
                    lines.push(format!("{symbol}={}", to_hex(r, g, b)));
                }
                DynamicColorKey::Rgb(r, g, b) => {
                    lines.push(format!("{symbol}={}", to_hex(*r, *g, *b)));
                }
            }
        }

        lines
    }
}

fn render_character_grid(screen: &vt100::Screen, rows: u16, cols: u16) -> String {
    let mut lines = Vec::with_capacity(usize::from(rows));
    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            line.push_str(&character_for_cell(screen.cell(row, col)));
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn render_color_grid(
    screen: &vt100::Screen,
    rows: u16,
    cols: u16,
    layer: SnapshotColorLayer,
    use_smart: bool,
    codebook: &mut ColorCodebook,
) -> Result<String> {
    let mut lines = Vec::with_capacity(usize::from(rows));
    for row in 0..rows {
        let mut line = String::with_capacity(usize::from(cols));
        for col in 0..cols {
            let selected = select_color_for_layer(screen.cell(row, col), layer, use_smart);
            line.push(codebook.symbol_for(selected)?);
        }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn character_for_cell(cell: Option<&vt100::Cell>) -> String {
    let Some(cell) = cell else {
        return " ".to_string();
    };

    if cell.is_wide_continuation() || !cell.has_contents() {
        " ".to_string()
    } else {
        cell.contents().to_string()
    }
}

fn select_color_for_layer(
    cell: Option<&vt100::Cell>,
    layer: SnapshotColorLayer,
    use_smart: bool,
) -> Color {
    let Some(cell) = cell else {
        return Color::Default;
    };

    let (foreground, background) = final_visible_colors(cell);
    match layer {
        SnapshotColorLayer::Foreground => {
            if use_smart && !cell.has_contents() {
                background
            } else {
                foreground
            }
        }
        SnapshotColorLayer::Background => background,
    }
}

fn final_visible_colors(cell: &vt100::Cell) -> (Color, Color) {
    let mut foreground = cell.fgcolor();
    let mut background = cell.bgcolor();
    if cell.inverse() {
        std::mem::swap(&mut foreground, &mut background);
    }
    (foreground, background)
}

fn dynamic_symbol_pool() -> Vec<char> {
    let reserved: HashSet<char> = ANSI_SLOT_SYMBOLS.iter().copied().collect();
    DYNAMIC_SYMBOL_SOURCE
        .chars()
        .filter(|symbol| !reserved.contains(symbol))
        .collect()
}

fn xterm_index_to_rgb(index: u8) -> (u8, u8, u8) {
    if index < 16 {
        return (0, 0, 0);
    }

    if index < 232 {
        let cube = index - 16;
        let red = cube / 36;
        let green = (cube % 36) / 6;
        let blue = cube % 6;
        let table = [0, 95, 135, 175, 215, 255];
        (
            table[usize::from(red)],
            table[usize::from(green)],
            table[usize::from(blue)],
        )
    } else {
        let shade = 8 + (index - 232) * 10;
        (shade, shade, shade)
    }
}

fn to_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
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
    use crate::protocol::{SnapshotColorMode, SnapshotTheme};

    fn split_layers_and_legend(output: &str) -> (Vec<&str>, Vec<&str>) {
        let (layers, legend) = output.split_once("\n\nLegend:\n").expect("missing legend");
        (layers.split("\n\n").collect(), legend.lines().collect())
    }

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
    fn viewport_ansi_text_contains_escape_sequences_for_styles() {
        let mut screen = ScreenBuffer::new(2, 1);
        screen.apply(b"\x1b[31mA");
        let output = screen.viewport_ansi_text();

        assert!(output.contains('A'));
        assert!(output.contains("\u{1b}["));
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

    #[test]
    fn foreground_layer_aligns_cell_by_cell_with_character_layer() {
        let mut screen = ScreenBuffer::new(4, 1);
        screen.apply(b"\x1b[31mAB");
        let output = screen
            .viewport_color_text(SnapshotColorMode::Foreground, SnapshotTheme::OneHalfDark)
            .unwrap();
        let (layers, legend) = split_layers_and_legend(&output);

        assert_eq!(layers[0], "AB  ");
        assert_eq!(layers[1], "rr..");
        assert!(legend.contains(&".=default"));
        assert!(legend.contains(&"r=ansi1(#e06c75)"));
    }

    #[test]
    fn smart_uses_background_for_empty_cells_and_foreground_for_non_empty_cells() {
        let mut screen = ScreenBuffer::new(3, 1);
        screen.apply(b"\x1b[44m\x1b[2J\x1b[H\x1b[31mA");
        let output = screen
            .viewport_color_text(SnapshotColorMode::Smart, SnapshotTheme::OneHalfDark)
            .unwrap();
        let (layers, _) = split_layers_and_legend(&output);

        assert_eq!(layers[0], "A  ");
        assert_eq!(layers[1], "rbb");
    }

    #[test]
    fn inverse_cells_export_final_visible_colors() {
        let mut screen = ScreenBuffer::new(2, 1);
        screen.apply(b"\x1b[31;44;7mI");
        let foreground = screen
            .viewport_color_text(SnapshotColorMode::Foreground, SnapshotTheme::OneHalfDark)
            .unwrap();
        let background = screen
            .viewport_color_text(SnapshotColorMode::Background, SnapshotTheme::OneHalfDark)
            .unwrap();
        let (fg_layers, _) = split_layers_and_legend(&foreground);
        let (bg_layers, _) = split_layers_and_legend(&background);

        assert_eq!(fg_layers[1], "b.");
        assert_eq!(bg_layers[1], "r.");
    }

    #[test]
    fn wide_characters_keep_layer_alignment_with_continuation_cells() {
        let mut screen = ScreenBuffer::new(3, 1);
        screen.apply("\x1b[32m你".as_bytes());
        let output = screen
            .viewport_color_text(SnapshotColorMode::Foreground, SnapshotTheme::OneHalfDark)
            .unwrap();
        let (layers, _) = split_layers_and_legend(&output);

        assert_eq!(layers[0], "你  ");
        assert_eq!(layers[1].chars().count(), 3);
        assert_eq!(layers[1].chars().next(), Some('g'));
    }

    #[test]
    fn color_mode_preserves_trailing_spaces_in_character_layer() {
        let mut screen = ScreenBuffer::new(5, 1);
        screen.apply(b"ab");
        let output = screen
            .viewport_color_text(SnapshotColorMode::Foreground, SnapshotTheme::OneHalfDark)
            .unwrap();
        let (layers, _) = split_layers_and_legend(&output);

        assert_eq!(layers[0], "ab   ");
    }

    #[test]
    fn legend_is_stable_for_default_ansi_indexed_and_rgb_colors() {
        let mut screen = ScreenBuffer::new(4, 1);
        screen.apply(b"d");
        screen.apply(b"\x1b[32ma");
        screen.apply(b"\x1b[38;5;17mb");
        screen.apply(b"\x1b[38;2;1;2;3mc");
        let output = screen
            .viewport_color_text(SnapshotColorMode::Foreground, SnapshotTheme::OneHalfDark)
            .unwrap();
        let (layers, legend) = split_layers_and_legend(&output);

        assert_eq!(layers[1], ".gAD");
        assert_eq!(legend[0], ".=default");
        assert_eq!(legend[1], "g=ansi2(#98c379)");
        assert_eq!(legend[2], "A=#00005f");
        assert_eq!(legend[3], "D=#010203");
    }

    #[test]
    fn foreground_background_mode_always_emits_fixed_layer_order() {
        let mut screen = ScreenBuffer::new(2, 1);
        screen.apply(b"\x1b[31;44mA");
        let output = screen
            .viewport_color_text(
                SnapshotColorMode::ForegroundBackground,
                SnapshotTheme::OneHalfDark,
            )
            .unwrap();
        let (layers, _) = split_layers_and_legend(&output);

        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], "A ");
        assert_eq!(layers[1], "r.");
        assert_eq!(layers[2], "b.");
    }

    #[test]
    fn dynamic_symbol_pool_exhaustion_returns_error() {
        let mut screen = ScreenBuffer::new(37, 1);
        let mut sequence = String::new();
        for index in 16u8..=52 {
            sequence.push_str(&format!("\x1b[38;5;{index}mX"));
        }
        screen.apply(sequence.as_bytes());

        let error = screen
            .viewport_color_text(SnapshotColorMode::Foreground, SnapshotTheme::OneHalfDark)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("supports up to 36 dynamic colors"),
            "unexpected error: {error:#}",
        );
    }
}
