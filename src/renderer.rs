use crate::app_state::EditorState;
use fontdue::{Font, FontSettings};
use pixels::Pixels;
use std::collections::HashMap;

const FONT_SIZE: f32 = 20.0;
const TEXT_X: i32 = 20;
const TEXT_Y: i32 = 40;
const BG_COLOR: [u8; 4] = [20, 22, 26, 255];
const FG_COLOR: [u8; 4] = [232, 234, 239, 255];

fn load_font() -> Font {
    let font_path = "/Users/sidpanda/Library/Fonts/Source Code Pro for Powerline.otf";
    if let Ok(bytes) = std::fs::read(font_path) {
        if let Ok(font) = Font::from_bytes(bytes, FontSettings::default()) {
            return font;
        }
    }

    panic!("No usable macOS system font found for fontdue rendering");
}

pub struct Renderer {
    font: Font,
    ch_glyph_cache: HashMap<char, (fontdue::Metrics, Vec<u8>)>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { font: load_font(), ch_glyph_cache: HashMap::new() }
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
        let cursor = if state.cursor_visible { "|" } else { " " };

        self.draw_text(
            frame,
            width,
            height,
            &editor_text,
            TEXT_X,
            TEXT_Y,
            FG_COLOR,
            cursor
        );
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
        mut x: i32,
        mut y: i32,
        color: [u8; 4],
        cursor: &str
    ) {
        let line_height = (FONT_SIZE as i32) + 8;

        for ch in text.chars() {
            if ch == '\n' {
                x = TEXT_X;
                y += line_height;
                continue;
            }

            let advance_width = self.draw_char_glyph(x, y, width, height, frame, color, ch);

            x += advance_width as i32;
        }

        if !cursor.is_empty() {
            self.draw_char_glyph(x, y, width, height, frame, color, cursor.chars().next().unwrap());
        }
        
    }

    /**
     * Draws a single character glyph onto the frame buffer at the specified position.
     * Returns the advance width of the character for positioning the next character.
     */
    fn draw_char_glyph(&mut self, x: i32, y: i32, width: u32, height: u32, frame: &mut [u8], color: [u8; 4], ch: char) -> f32 {
        let font = &self.font;
        let (metrics, bitmap) = self
            .ch_glyph_cache
            .entry(ch)
            .or_insert_with(|| font.rasterize(ch, FONT_SIZE));

        let metrics = *metrics;
        let bitmap = bitmap.as_slice();

        let cursor_x = x + metrics.xmin;
        let cursor_y = y + metrics.ymin;

        for by in 0..metrics.height {
            for bx in 0..metrics.width {
                let px = cursor_x + bx as i32;
                let py = cursor_y + by as i32;
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
        return metrics.advance_width
    }
}