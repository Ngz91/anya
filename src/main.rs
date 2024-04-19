use arboard::Clipboard;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use errors::CustomError;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

pub mod app;
pub mod errors;
pub mod layout;
pub mod requester;
pub mod utils;

use crate::layout::MainLayout;

use app::App;
use requester::Requester;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Running,
    Exit,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // let (state_tx, state_rx) = mpsc::channel::<State>(1);
    // let (request_tx, request_rx) = mpsc::channel::<reqwest::RequestBuilder>(1);
    // let (response_tx, response_rx) = mpsc::channel::<Result<serde_json::Value, CustomError>>(1);

    let mut clipboard = Clipboard::new().unwrap();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    // let mut requester = Requester::new(state_rx, request_rx, response_tx, None);
    let client = reqwest::Client::new();

    app.activate_deactivate_textarea();

    // tokio::spawn(async move { requester.start_requester().await });
    let res = app.run_app(&mut terminal, &client, &mut clipboard).await;

    // let _ = tokio::join!(
    //     requester.start_requester(),
    //     app.run_app(&mut terminal, &client, &mut clipboard, state_tx.clone()),
    // );

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error encountered: {err:?}")
    }

    Ok(())
}
