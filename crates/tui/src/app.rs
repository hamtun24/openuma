use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub struct App {
    pub should_quit: bool,
    pub title: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            title: "OpenUMA TUI".to_string(),
        }
    }

    pub fn update(&mut self) {
        if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.should_quit = true;
                        }
                        KeyCode::Char('r') => {
                            // refresh
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
