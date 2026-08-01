use std::time::{Duration, Instant};
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

const BLINK_INTERVAL: Duration = Duration::from_millis(300);

pub struct EditorState {
    pub text: String,
    pub cursor_visible: bool,
    last_blink: Instant,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_visible: true,
            last_blink: Instant::now(),
        }
    }

    pub fn next_blink_deadline(&self) -> Instant {
        self.last_blink + BLINK_INTERVAL
    }

    pub fn tick_blink(&mut self, now: Instant) -> bool {
        if now >= self.next_blink_deadline() {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink = now;
            return true;
        }

        false
    }

    pub fn handle_command_key(&mut self, input: &KeyEvent) -> String {
        let key_name = format!("{:?}", input.logical_key);

        if matches!(input.logical_key, Key::Named(NamedKey::Backspace)) {
            self.text.pop();
        }

        if matches!(input.logical_key, Key::Named(NamedKey::Enter)) {
            self.text.push('\n');
        }

        if matches!(input.logical_key, Key::Named(NamedKey::Tab)) {
            self.text.push_str("    ");
        }

        if matches!(input.logical_key, Key::Named(NamedKey::Space)) {
            self.text.push_str(" ");
        }

        key_name
    }

    pub fn push_character(&mut self, ch: char) -> bool {
        if ch.is_control() {
            return false;
        }

        self.text.push(ch);
        true
    }
}
