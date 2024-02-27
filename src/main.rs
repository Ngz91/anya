#![allow(dead_code)]

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use ratatui_textarea::{Input, Key};
use std::io;

pub mod app;
pub mod layout;

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

    loop {
        terminal.draw(|f| {
            let layout = MainLayout::new(f);
            app.render_ui(f, &layout);
        })?;

        match crossterm::event::read()?.into() {
            Input {key: Key::Esc, ..} => break,
            Input {
                key: Key::Char('g'),
                ctrl: true,
                ..
            } => {
                let resp = app.get_response(&client);
                app.set_response(resp)
            }
            input => {
                todo!()
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
