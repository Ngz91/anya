use arboard::Clipboard;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use local_types::ResultSerde;
use ratatui::prelude::{CrosstermBackend, Terminal};
use requester::listen_requests;
use std::io;
use tokio::sync::{mpsc, watch};

pub mod app;
pub mod errors;
pub mod layout;
pub mod local_types;
pub mod requester;
pub mod utils;

use crate::layout::MainLayout;

use app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (cancel_send, cancel_watch) = watch::channel(false);
    let (response_tx, response_rx) = mpsc::unbounded_channel::<ResultSerde>();
    let (request_tx, request_rx) = mpsc::unbounded_channel::<reqwest::RequestBuilder>();

    let mut clipboard = Clipboard::new().unwrap();
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new();
    let client = reqwest::Client::new();

    app.activate_deactivate_textarea();

    tokio::spawn(listen_requests(cancel_watch, request_rx, response_tx));
    let res = app
        .run_app(
            &mut terminal,
            &client,
            &mut clipboard,
            cancel_send,
            request_tx,
            response_rx,
        )
        .await;

    io::stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    disable_raw_mode()?;

    if let Err(err) = res {
        println!("Error encountered: {err:?}")
    }

    Ok(())
}
