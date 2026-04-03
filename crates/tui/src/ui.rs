use crate::state::AppState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, Paragraph},
};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("OpenUMA - Hardware Profiler")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Welcome"));
    frame.render_widget(title, chunks[0]);

    if state.is_loading {
        let loading = Paragraph::new("Loading hardware profile...")
            .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(loading, chunks[1]);
    } else if let Some(ref profile) = state.profile {
        let cpu_info = format!(
            "CPU: {} ({} cores, {} threads)",
            profile.cpu.model, profile.cpu.cores, profile.cpu.threads
        );
        let ram_info = format!(
            "RAM: {} MB total, {} MB available",
            profile.ram.total_bytes / (1024 * 1024),
            profile.ram.available_bytes / (1024 * 1024)
        );
        let platform_info = format!(
            "Platform: {} ({})",
            profile.platform.os, profile.platform.compute_backend
        );

        let info = vec![cpu_info, ram_info, platform_info];
        let list = List::new(info).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Hardware Profile"),
        );
        frame.render_widget(list, chunks[1]);
    } else if let Some(ref error) = state.error {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        frame.render_widget(error_msg, chunks[1]);
    } else {
        let help = Paragraph::new("Press 'r' to refresh hardware info")
            .block(Block::default().borders(Borders::ALL).title("Info"));
        frame.render_widget(help, chunks[1]);
    }

    let footer = Paragraph::new("Press 'q' or Esc to quit | 'r' to refresh")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);
}
