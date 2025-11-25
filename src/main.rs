mod app;
mod functions;
mod ui;

use crate::ui::*;
use app::*;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let app = App::new()?;
    let result = run_app(&mut terminal, app);
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode() {
                Mode::FileSelection => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next_item(),
                    KeyCode::Up => app.previous_item(),
                    KeyCode::Enter => app.start_field_selection(),
                    _ => {}
                },
                Mode::FieldSelection => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('b') => app.back_to_files(),
                    KeyCode::Down => app.next_item(),
                    KeyCode::Up => app.previous_item(),
                    KeyCode::Enter => app.start_editing(),
                    _ => {}
                },
                Mode::Editing => match key.code {
                    KeyCode::Enter => {
                        if let Err(e) = app.finish_editing() {
                            app.set_message(format!("Error: {}", e));
                        }
                    }
                    KeyCode::Esc => app.cancel_editing(),
                    KeyCode::Char(c) => app.push_to_buffer(c),
                    KeyCode::Backspace => {
                        app.pop_from_buffer();
                    }
                    _ => {}
                },
            }
        }
    }
}
