use std::{io, time::Duration};

use arboard::Clipboard;
use crossterm::event::{self, Event};
use ratatui::{
    backend::Backend,
    style::Stylize,
    widgets::{Block, Borders},
    Frame, Terminal,
};
use tokio::{
    select,
    sync::{mpsc, watch},
};
use tui_textarea::{Input, Key, TextArea};

use crate::{errors, local_types::ResultSerde, utils, MainLayout};

// async fn make_request(
//     client_builder: Result<reqwest::RequestBuilder, errors::CustomError>,
// ) -> std::result::Result<serde_json::Value, errors::CustomError> {
//     match client_builder {
//         Ok(client) => {
//             let resp = client.send().await?.json::<serde_json::Value>().await?;
//             Ok(resp)
//         }
//         Err(err) => Err(err),
//     }
// }

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    Running,
    Exit,
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
        cancel_send: watch::Sender<bool>,
        request_tx: mpsc::UnboundedSender<reqwest::RequestBuilder>,
        mut response_rx: mpsc::UnboundedReceiver<ResultSerde>,
    ) -> io::Result<()> {
        self.textarea[0].insert_str("https://");
        let tick_rate = Duration::from_millis(250);

        loop {
            terminal.draw(|f| {
                let layout = MainLayout::new(f);
                self.render_ui(f, &layout);
            })?;
            if crossterm::event::poll(tick_rate)? {
                if let Event::Key(key_event) = event::read().unwrap() {
                    let input = key_event.into();

                    match input {
                        Input { key: Key::Esc, .. }
                        | Input {
                            key: Key::Char('q'),
                            ctrl: true,
                            ..
                        } => {
                            let _ = cancel_send.send(true);
                            self.state = State::Exit
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
                            let client_builder = self.build_client(client, reqwest::Method::GET);
                            // tokio::select! {
                            //     response = make_request(client_builder) => {
                            //         self.set_response(response)
                            //     }
                            // }
                            let _ = request_tx.send(client_builder.unwrap());
                        }
                        // POST method
                        Input {
                            key: Key::Char('h'),
                            ctrl: true,
                            ..
                        } => {
                            let client_builder = self.build_client(client, reqwest::Method::POST);
                            // tokio::select! {
                            //     response = make_request(client_builder) => {
                            //         self.set_response(response)
                            //     }
                            // }
                            let _ = request_tx.send(client_builder.unwrap());
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
                        _ => self.render_inputs(input),
                    }

                    if self.state == State::Exit {
                        return Ok(());
                    }
                }
            }
        }
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

    async fn set_response(&mut self, mut response_rx: mpsc::UnboundedReceiver<ResultSerde>) {
        self.response = match response_rx.recv().await.unwrap() {
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

    fn render_inputs(&mut self, input: Input) {
        self.textarea[self.which].input(input);
    }
}
