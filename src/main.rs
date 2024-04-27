use arboard::Clipboard;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io;

pub mod app;
pub mod errors;
pub mod layout;
pub mod utils;

use crate::layout::MainLayout;

use app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut clipboard = Clipboard::new().unwrap();

    let mut stdout = io::stdout();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let client = reqwest::Client::new();

    app.activate_deactivate_textarea();
    app.run_app(&mut terminal, &client, &mut clipboard).await?;

    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    disable_raw_mode()?;

    Ok(())
}
