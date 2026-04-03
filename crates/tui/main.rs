use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tui::state::AppState;
use tui::ui::draw;
use tui::App;

fn main() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut app = App::new();
    let mut state = AppState::new();

    state.set_loading(true);
    state.set_profile(hw_probe::probe_all()?);

    loop {
        terminal.draw(|f| draw(f, &state))?;
        app.update();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
