#![allow(dead_code)]

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui_textarea::{Input, Key};
use std::io;

pub mod app;
pub mod layout;
pub mod utils;

use crate::layout::MainLayout;

fn main() -> std::io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::default();
    let client = reqwest::Client::new();

    app.activate_deactivate_textarea();

    loop {
        terminal.draw(|f| {
            let layout = MainLayout::new(f);
            app.render_ui(f, &layout);
        })?;

        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            Input {
                key: Key::Char('g'),
                ctrl: true,
                ..
            } => {
                let resp = app.get_response(&client);
                app.set_response(resp)
            }
            Input {
                key: Key::Char('x'),
                ctrl: true,
                ..
            } => {
                app.handle_textarea_events();
            }
            input => {
                app.handle_inputs(input);
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    Ok(())
}
