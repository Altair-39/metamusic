use crate::app::App;
use crate::app::Mode;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use ratatui_image::StatefulImage;
use std::{error::Error, io};

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
                Constraint::Percentage(60),
                Constraint::Percentage(40),
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

    // Split the files area horizontally for files list and content
    let files_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
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

    // Right side: Split into tags and album art
    let right_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(files_chunks[1]);

    // Tags preview (left side of right panel)
    let tags_preview = create_tags_preview_widget(app);
    f.render_widget(tags_preview, right_chunks[0]);

    // Album art (right side of right panel)
    create_album_art_widget(f, app, right_chunks[1]);

    // Bottom panel - different content based on mode()
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

fn create_tags_preview_widget(app: &App) -> Paragraph<'static> {
    if let Some(current_file) = app.files().get(app.selected_file()) {
        if let Some(tag_info) = app.tags_for_file(current_file) {
            let mut lines = Vec::new();

            // Album art status
            let has_art = app.has_album_art(current_file);
            let art_status_text = if has_art {
                "✓ Album Art".to_string()
            } else {
                "✗ No Album Art".to_string()
            };

            lines.push(Line::from(Span::styled(
                art_status_text,
                if has_art {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            )));
            lines.push(Line::from(""));

            // Tag information
            let tag_content = vec![
                ("Title".to_string(), tag_info.title.clone()),
                ("Artist".to_string(), tag_info.artist.clone()),
                ("Album".to_string(), tag_info.album.clone()),
                ("Year".to_string(), tag_info.year.clone()),
                ("Track".to_string(), tag_info.track.clone()),
            ];

            for (field, value) in tag_content {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:<8}: ", field),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(value, Style::default().fg(Color::White)),
                ]));
            }

            Paragraph::new(lines)
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
    }
}

fn create_album_art_widget(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = Block::default().borders(Borders::ALL).title("🎵 Album Art");

    // Draw the block first
    f.render_widget(&block, area);

    // Inner area for the image
    let inner_area = block.inner(area);

    if inner_area.width < 3 || inner_area.height < 3 {
        // Area too small for meaningful image display
        let warning = Paragraph::new("Area too small")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(warning, inner_area);
        return;
    }

    let current_file = app.files().get(app.selected_file()).cloned();

    if let Some(current_file) = current_file {
        if let Some(protocol_arc) = app.load_album_art(&current_file) {
            if let Ok(mut protocol) = protocol_arc.lock() {
                // Create a centered area within the inner area
                let centered_area = center_area(inner_area);

                let image_widget = StatefulImage::default();
                f.render_stateful_widget(image_widget, centered_area, &mut *protocol);

                if let Some(Err(e)) = protocol.last_encoding_result() {
                    let error_msg = Paragraph::new(format!("Render error: {}", e))
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center);
                    f.render_widget(error_msg, inner_area);
                }
            }
        } else {
            show_album_art_placeholder(f, inner_area);
        }
    } else {
        show_album_art_placeholder(f, inner_area);
    }
}

fn center_area(area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let max_width = area.width;
    let max_height = area.height;

    let target_width = max_width.saturating_sub(2); // Leave some margin
    let target_height = max_height.saturating_sub(2); // Leave some margin

    let x = area.x + (max_width.saturating_sub(target_width)) / 2;
    let y = area.y + (max_height.saturating_sub(target_height)) / 2;

    ratatui::layout::Rect {
        x,
        y,
        width: target_width,
        height: target_height,
    }
}

fn show_album_art_placeholder(f: &mut Frame, area: ratatui::layout::Rect) {
    let placeholder_content = vec![
        Line::from(""),
        Line::from("╭───────────╮"),
        Line::from("│           │"),
        Line::from("│    ___    │"),
        Line::from("│   /   \\   │"),
        Line::from("│  | 📀 |   │"),
        Line::from("│   \\___/   │"),
        Line::from("│           │"),
        Line::from("╰───────────╯"),
        Line::from(""),
    ];

    let placeholder = Paragraph::new(placeholder_content)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(placeholder, area);
}
