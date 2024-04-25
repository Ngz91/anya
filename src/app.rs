use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use arboard::Clipboard;
use ratatui::{
    backend::Backend,
    style::Stylize,
    widgets::{Block, Borders},
    Frame, Terminal,
};
use tokio::sync::mpsc;
use tui_textarea::{Input, Key, TextArea};

use crate::{errors, utils, MainLayout, State};

async fn testing_making_req(
    client_builder: reqwest::RequestBuilder,
    response_tx: mpsc::Sender<Result<serde_json::Value, errors::CustomError>>,
    response_sender: Arc<Mutex<tokio::sync::watch::Sender<serde_json::Value>>>,
) -> std::result::Result<(), errors::CustomError> {
    let resp = client_builder
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let _ = response_tx.send(Ok(resp.clone())).await;
    let _ = response_sender.lock().unwrap().send(resp.clone());

    drop(response_sender);
    response_tx.closed().await;

    Ok(())
}

#[derive(Default)]
pub struct App<'a> {
    response: Option<serde_json::Value>,
    textarea: [TextArea<'a>; 2],
    which: usize,
    state: State,
}

impl App<'_> {
    pub fn new() -> Self {
        App::default()
    }

    pub async fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        client: &reqwest::Client,
        clipboard: &mut Clipboard,
    ) -> io::Result<()> {
        self.textarea[0].insert_str("https://");
        let (response_tx, mut response_rx) =
            mpsc::channel::<Result<serde_json::Value, errors::CustomError>>(1);
        let response_sender = Arc::new(Mutex::new(tokio::sync::watch::Sender::new(
            serde_json::json!({}),
        )));
        let mut response_watcher = response_sender.lock().unwrap().subscribe();

        loop {
            terminal.draw(|f| {
                let layout = MainLayout::new(f);
                self.render_ui(f, &layout);
            })?;

            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => {
                    // self.state = State::Exit;
                    // state_tx.send(self.state.clone()).await.unwrap();
                    response_rx.close();
                    return Ok(());
                }
                Input {
                    key: Key::Char('x'),
                    ctrl: true,
                    ..
                } => {
                    self.change_textarea();
                }
                // GET method
                Input {
                    key: Key::Char('g'),
                    ctrl: true,
                    ..
                } => {
                    // let resp = self.request(client, reqwest::Method::GET).await;
                    // self.set_response(resp)
                    let local_tx = response_tx.clone();
                    let test_tx = Arc::clone(&response_sender);
                    let client_builder = self.build_client(client, reqwest::Method::GET).unwrap();
                    tokio::spawn(async move {
                        testing_making_req(client_builder, local_tx, test_tx).await
                    });
                }
                // POST method
                Input {
                    key: Key::Char('h'),
                    ctrl: true,
                    ..
                } => {
                    // let resp = self.request(client, reqwest::Method::POST).await;
                    // self.set_response(resp)

                    let local_tx = response_tx.clone();
                    let test_tx = Arc::clone(&response_sender);
                    let client_builder = self.build_client(client, reqwest::Method::POST).unwrap();
                    tokio::spawn(async move {
                        testing_making_req(client_builder, local_tx, test_tx).await
                    });
                }
                // Paste clipboard contents into the active textarea
                Input {
                    key: Key::Char('l'),
                    ctrl: true,
                    ..
                } => {
                    let clip_text = clipboard.get_text().unwrap();
                    self.textarea[self.which].insert_str(clip_text);
                }
                // Select textarea contents
                Input {
                    key: Key::Char('k'),
                    ctrl: true,
                    ..
                } => {
                    self.textarea[self.which].select_all();
                }
                input => {
                    self.handle_inputs(input);
                }
            }
            match response_watcher.has_changed() {
                Ok(received_data) => {
                    if received_data {
                        self.test_set_response(response_watcher.borrow_and_update().clone())
                    }
                }
                Err(_e) => {}
            }
            // self.set_response(response_rx.try_recv().unwrap())
        }
    }

    fn test_set_response(&mut self, response: serde_json::Value) {
        self.response = Some(response);
    }
    fn render_ui(&mut self, f: &mut Frame, layout: &MainLayout) {
        self.textarea[0].set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Request Url")
                .bold()
                .light_blue(),
        );
        self.textarea[1].set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Json")
                .bold()
                .light_blue(),
        );
        let response_block = Block::default()
            .borders(Borders::ALL)
            .title("Response")
            .bold()
            .light_green();

        f.render_widget(self.textarea[0].widget(), layout.request_layout[0]);
        f.render_widget(self.textarea[1].widget(), layout.request_layout[1]);
        f.render_widget(response_block, layout.response_layout[0]);

        if let Some(resp) = &self.response {
            let resp = serde_json::to_string_pretty(resp).unwrap();
            let response_paragraph = utils::create_text(&resp, vec![2, 2, 1, 2]);
            f.render_widget(response_paragraph, layout.response_layout[0])
        }
    }

    pub fn activate_deactivate_textarea(&mut self) {
        utils::activate(&mut self.textarea[0]);
        utils::deactivate(&mut self.textarea[1]);
    }

    async fn request(
        &mut self,
        client: &reqwest::Client,
        method: reqwest::Method,
    ) -> std::result::Result<serde_json::Value, errors::CustomError> {
        let request_url = &self.textarea[0].lines()[0];

        let has_json = !self.textarea[1].lines()[0].is_empty();

        let mut request_builder = client
            .request(method, request_url)
            .timeout(Duration::from_secs(5));

        match has_json {
            true => {
                let request_json = &self.textarea[1].lines().join("");
                let json_value: serde_json::Value = match serde_json::from_str(request_json) {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(errors::handle_serde_json_error(err));
                    }
                };
                request_builder = request_builder.json(&json_value);
            }
            false => {} // Do nothing, no JSON
        }

        let resp = request_builder
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(resp)
    }

    fn build_client(
        &mut self,
        client: &reqwest::Client,
        method: reqwest::Method,
    ) -> Result<reqwest::RequestBuilder, errors::CustomError> {
        let request_url = &self.textarea[0].lines()[0];

        let has_json = !self.textarea[1].lines()[0].is_empty();

        let mut request_builder = client
            .request(method, request_url)
            .timeout(Duration::from_secs(5));

        match has_json {
            true => {
                let request_json = &self.textarea[1].lines().join("");
                let json_value: serde_json::Value = match serde_json::from_str(request_json) {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(errors::handle_serde_json_error(err));
                    }
                };
                request_builder = request_builder.json(&json_value);
            }
            false => {} // Do nothing, no JSON
        }

        Ok(request_builder)
    }

    fn set_response(
        &mut self,
        response: std::result::Result<serde_json::Value, errors::CustomError>,
    ) {
        self.response = match response {
            Ok(resp) => Some(resp),
            Err(err) => Some(serde_json::json!({
                "Request error": err.to_string()
            })),
        };
    }

    fn change_textarea(&mut self) {
        utils::deactivate(&mut self.textarea[self.which]);
        self.which = (self.which + 1) % 2;
        utils::activate(&mut self.textarea[self.which])
    }

    fn handle_inputs(&mut self, input: Input) {
        self.textarea[self.which].input(input);
    }
}
