mod app;
mod functions;
mod ui;

use crate::ui::ui;
use app::*;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::error::Error;
use std::io;

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

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
                    KeyCode::Char('b') => app.back_to_files(),
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
