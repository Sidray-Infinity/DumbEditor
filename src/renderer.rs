use crate::app_state::EditorState;
use fontdue::{Font, FontSettings};
use pixels::Pixels;
use std::collections::HashMap;

const FONT_SIZE: f32 = 25.0;
const TEXT_X: i32 = 20;
const TEXT_Y: i32 = 20;
const BG_COLOR: [u8; 4] = [20, 22, 26, 255];
const FG_COLOR: [u8; 4] = [232, 234, 239, 255];
const GRID_COLOR: [u8; 4] = [50, 55, 64, 255];
const DEBUG_CELL_GRID: bool = false;
const SAMPLE_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789|!@#$%^&*()_+-=[]{};:'\",.<>/?\\`~";
const CELL_PADDING: i32 = 2;

fn load_font() -> Font {
    let font_path = "/System/Library/Fonts/Menlo.ttc";
    if let Ok(bytes) = std::fs::read(font_path) {
        if let Ok(font) = Font::from_bytes(bytes, FontSettings::default()) {
            return font;
        }
    }

    panic!("No usable macOS system font found for fontdue rendering");
}

pub struct Renderer {
    font: Font,
    cell_width: i32,
    cell_height: i32,
    ch_glyph_cache: HashMap<char, (fontdue::Metrics, Vec<u8>)>,
}

impl Renderer {
    pub fn new() -> Self {
        let font = load_font();
        let mut max_advance = 0.0f32;
        let mut max_bitmap_width = 0.0f32;
        let mut max_bitmap_height = 0.0f32;

        for ch in SAMPLE_CHARS.chars() {
            let (metrics, _) = font.rasterize(ch, FONT_SIZE);
            max_advance = max_advance.max(metrics.advance_width);
            max_bitmap_width = max_bitmap_width.max(metrics.width as f32);
            max_bitmap_height = max_bitmap_height.max(metrics.height as f32);
        }

        let mut cell_width = max_advance
            .ceil()
            .max(max_bitmap_width.ceil()) as i32
            + CELL_PADDING;

        cell_width = cell_width.max(1);

        let mut cell_height = if let Some(line) = font.horizontal_line_metrics(FONT_SIZE) {
            let line_height = (line.ascent - line.descent + line.line_gap).ceil() as i32;
            line_height.max(max_bitmap_height.ceil() as i32 + CELL_PADDING)
        } else {
            let fallback_height = max_bitmap_height.ceil() as i32;
            fallback_height + CELL_PADDING
        };

        cell_height = cell_height.max(1);

        Self {
            font,
            cell_width,
            cell_height,
            ch_glyph_cache: HashMap::new(),
        }
    }

    /**
     * Draws the current state of the editor onto the pixel buffer.
     * Clears the buffer, draws the text, and handles cursor visibility.
     */
    pub fn draw(&mut self, pixels: &mut Pixels, state: &EditorState) {
        let size = pixels.texture().size();
        let width = size.width;
        let height = size.height;
        let frame = pixels.frame_mut();

        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }

        let editor_text = &state.text;
        let show_cursor = state.cursor_visible;

        self.draw_text(
            frame,
            width,
            height,
            &editor_text,
            TEXT_X,
            TEXT_Y,
            FG_COLOR,
            show_cursor,
        );

        if DEBUG_CELL_GRID {
            self.draw_cell_grid(frame, width, height, TEXT_X, TEXT_Y);
        }
    }

    /**
     * Draws the given text onto the frame buffer at the specified position.
     * Handles newlines and draws the cursor if provided.
     */
    fn draw_text(
        &mut self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        text: &str,
        origin_x: i32,
        origin_y: i32,
        color: [u8; 4],
        show_cursor: bool,
    ) {
        let mut col = 0i32;
        let mut row = 0i32;

        for ch in text.chars() {
            if ch == '\n' {
                col = 0;
                row += 1;
                continue;
            }

            let cell_x = origin_x + col * self.cell_width;
            let row_top = origin_y + row * self.cell_height;
            self.draw_char_glyph(cell_x, row_top, width, height, frame, color, ch);
            col += 1;
        }

        if show_cursor {
            let cell_x = origin_x + col * self.cell_width;
            let row_top = origin_y + row * self.cell_height;
            self.draw_char_glyph(cell_x, row_top, width, height, frame, color, '|');
        }
        
    }

    /**
     * Draws a single character glyph onto the frame buffer at the specified position.
     * Returns the advance width of the character for positioning the next character.
     */
    fn draw_char_glyph(&mut self, cell_x: i32, row_top: i32, width: u32, height: u32, frame: &mut [u8], color: [u8; 4], ch: char) {
        let font = &self.font;
        let (metrics, bitmap) = self
            .ch_glyph_cache
            .entry(ch)
            .or_insert_with(|| font.rasterize(ch, FONT_SIZE));

        let metrics = *metrics;
        let bitmap = bitmap.as_slice();

        let glyph_x = cell_x + ((self.cell_width - metrics.width as i32).max(0) / 2);
        let glyph_y = row_top + self.cell_height - CELL_PADDING - metrics.height as i32;

        for by in 0..metrics.height {
            for bx in 0..metrics.width {
                let px = glyph_x + bx as i32;
                let py = glyph_y + by as i32;
                if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                    continue;
                }

                let bitmap_alpha = bitmap[by * metrics.width + bx];
                if bitmap_alpha == 0 {
                    continue;
                }

                let pixel_index = ((py as u32 * width + px as u32) * 4) as usize;
                frame[pixel_index] = color[0];
                frame[pixel_index + 1] = color[1];
                frame[pixel_index + 2] = color[2];
                frame[pixel_index + 3] = bitmap_alpha;
            }
        }
    }

    fn draw_cell_grid(&self, frame: &mut [u8], width: u32, height: u32, origin_x: i32, origin_y: i32) {
        if self.cell_width <= 0 || self.cell_height <= 0 {
            return;
        }

        let width_i32 = width as i32;
        let height_i32 = height as i32;

        let cols = ((width_i32 - origin_x).max(0) + self.cell_width - 1) / self.cell_width;
        let rows = ((height_i32 - origin_y).max(0) + self.cell_height - 1) / self.cell_height;

        for row in 0..=rows {
            let y = origin_y + row * self.cell_height;
            self.draw_hline(frame, width, height, origin_x, y, GRID_COLOR);
        }

        for col in 0..=cols {
            let x = origin_x + col * self.cell_width;
            self.draw_vline(frame, width, height, x, origin_y, GRID_COLOR);
        }
    }

    fn draw_hline(&self, frame: &mut [u8], width: u32, height: u32, x0: i32, y: i32, color: [u8; 4]) {
        if y < 0 || y >= height as i32 {
            return;
        }

        let start_x = x0.max(0);
        let end_x = width as i32 - 1;
        for x in start_x..=end_x {
            self.set_pixel(frame, width, x, y, color);
        }
    }

    fn draw_vline(&self, frame: &mut [u8], width: u32, height: u32, x: i32, y0: i32, color: [u8; 4]) {
        if x < 0 || x >= width as i32 {
            return;
        }

        let start_y = y0.max(0);
        let end_y = height as i32 - 1;
        for y in start_y..=end_y {
            self.set_pixel(frame, width, x, y, color);
        }
    }

    fn set_pixel(&self, frame: &mut [u8], width: u32, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 {
            return;
        }

        let pixel_index = ((y as u32 * width + x as u32) * 4) as usize;
        if pixel_index + 3 >= frame.len() {
            return;
        }

        frame[pixel_index] = color[0];
        frame[pixel_index + 1] = color[1];
        frame[pixel_index + 2] = color[2];
        frame[pixel_index + 3] = color[3];
    }
}