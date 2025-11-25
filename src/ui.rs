use crate::app::App;
use crate::app::Mode;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::Alignment;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::error::Error;
use std::io;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title = Paragraph::new("Metamusic - A Rust Tags Editor")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Split the files area horizontally for files list and tags preview
    let files_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Files list (left side)
    let file_items: Vec<ListItem> = app
        .files()
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app.selected_file() && app.mode() == &Mode::FileSelection {
                Style::default().fg(Color::Yellow)
            } else if i == app.selected_file() {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let display = if i == app.selected_file() {
                format!("▶ {}", file)
            } else {
                format!("  {}", file)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let files_list = List::new(file_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("MP3 Files (↑↓ to select)"),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut list_state = ListState::default().with_selected(Some(app.selected_file()));

    f.render_stateful_widget(files_list, files_chunks[0], &mut list_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalLeft)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(app.files().len()).position(app.selected_file());
    f.render_stateful_widget(scrollbar, files_chunks[0], &mut scrollbar_state);

    let tags_preview = if let Some(current_file) = app.files().get(app.selected_file()) {
        if let Some(tag_info) = app.tags_for_file(current_file) {
            let tag_content = vec![
                Line::from(vec![
                    Span::styled("Title:  ", Style::default().fg(Color::Yellow)),
                    Span::styled(tag_info.title, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Artist: ", Style::default().fg(Color::Yellow)),
                    Span::styled(tag_info.artist, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Album:  ", Style::default().fg(Color::Yellow)),
                    Span::styled(tag_info.album, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Year:   ", Style::default().fg(Color::Yellow)),
                    Span::styled(tag_info.year, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Track:  ", Style::default().fg(Color::Yellow)),
                    Span::styled(tag_info.track, Style::default().fg(Color::White)),
                ]),
            ];

            Paragraph::new(tag_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Tags: {}", current_file)),
                )
                .wrap(Wrap { trim: true })
        } else {
            Paragraph::new("Cannot read tags from this file")
                .block(Block::default().borders(Borders::ALL).title("Tags Preview"))
                .style(Style::default().fg(Color::Red))
        }
    } else {
        Paragraph::new("Select a file to view its tags")
            .block(Block::default().borders(Borders::ALL).title("Tags Preview"))
            .style(Style::default().fg(Color::Gray))
    };

    f.render_widget(tags_preview, files_chunks[1]);
    // Right panel - different content based on mode()
    match app.mode() {
        Mode::FileSelection => {
            let instructions = Paragraph::new("Press ENTER to select this file and edit its tags")
                .block(Block::default().borders(Borders::ALL).title("Instructions"))
                .wrap(Wrap { trim: true });
            f.render_widget(instructions, chunks[2]);
        }
        Mode::FieldSelection => {
            let field_items: Vec<ListItem> = app
                .fields()
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let style = if i == app.selected_field() {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(format!("✎ {}", field), style)))
                })
                .collect();

            let fields_list = List::new(field_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Fields to Edit (↑↓ to select, ENTER to edit)"),
                )
                .highlight_style(Style::default().bg(Color::DarkGray));
            f.render_widget(fields_list, chunks[2]);
        }
        Mode::Editing => {
            let input_display = if app.input_buffer().is_empty() {
                "_"
            } else {
                app.input_buffer()
            };

            let editing_panel = Paragraph::new(format!(
                "Editing {}:\n\n{}\n\nType new value and press ENTER to save",
                app.current_field()
                    .as_ref()
                    .unwrap_or(&&"Unknown".to_string()),
                input_display
            ))
            .block(Block::default().borders(Borders::ALL).title("Editing Mode"))
            .style(Style::default().fg(Color::Cyan))
            .wrap(Wrap { trim: true });
            f.render_widget(editing_panel, chunks[2]);
        }
    }

    // Status/Message bar
    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[3]);

    let mode_indicator = match app.mode() {
        Mode::FileSelection => " File Selection",
        Mode::FieldSelection => "✎ Field Selection",
        Mode::Editing => " Editing",
    };

    let mode_para = Paragraph::new(format!("{} | {}", mode_indicator, app.message()))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(mode_para, status_chunks[0]);

    let help_text = match app.mode() {
        Mode::FileSelection => "↑↓: Navigate | Enter: Select File | q: Quit",
        Mode::FieldSelection => "↑↓: Navigate | Enter: Edit Field | b: Back to Files | q: Quit",
        Mode::Editing => "Type: Edit | Enter: Save | Esc: Cancel | b: Back to Files",
    };

    let help_para = Paragraph::new(help_text).style(Style::default().fg(Color::Gray));
    f.render_widget(help_para, status_chunks[1]);
}
