use std::{io, time::Duration};

use arboard::Clipboard;
use crossterm::event;

use crate::{errors, local_types::ResultSerde, utils, MainLayout};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use tokio::sync::{mpsc, watch};
use tui_textarea::{Input, Key, TextArea};

#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    Idle,
    Running,
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
                if let event::Event::Key(key_event) = event::read().unwrap() {
                    if self.state == State::Idle {
                        let input = key_event.into();

                        match input {
                            Input { key: Key::Esc, .. }
                            | Input {
                                key: Key::Char('q'),
                                ctrl: true,
                                ..
                            } => {
                                let _ = cancel_send.send(true);
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
                                let client_builder_result =
                                    self.build_client(client, reqwest::Method::GET);
                                self.send_builder(client_builder_result, request_tx.clone());
                            }
                            // POST method
                            Input {
                                key: Key::Char('h'),
                                ctrl: true,
                                ..
                            } => {
                                let client_builder_result =
                                    self.build_client(client, reqwest::Method::POST);
                                self.send_builder(client_builder_result, request_tx.clone());
                            }
                            // Paste clipboard contents into the active textarea
                            Input {
                                key: Key::Char('l'),
                                ctrl: true,
                                ..
                            } => {
                                let clip_text_result = clipboard.get_text();
                                match clip_text_result {
                                    Ok(clip_text) => {
                                        self.textarea[self.which].insert_str(clip_text);
                                    }
                                    Err(_err) => {}
                                }
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
                    }
                }
            }
            match response_rx.try_recv() {
                Ok(res) => {
                    self.set_response(res);
                    self.state = State::Idle
                }
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => {}
            }
        }
    }

    fn send_builder(
        &mut self,
        builder_result: Result<reqwest::RequestBuilder, errors::CustomError>,
        request_tx: mpsc::UnboundedSender<reqwest::RequestBuilder>,
    ) {
        match builder_result {
            Ok(client_builder) => {
                let _ = request_tx.send(client_builder);
                self.state = State::Running;
            }
            Err(error) => {
                self.response = Some(serde_json::json!({"Request error": error.to_string()}))
            }
        }
        drop(request_tx);
    }

    pub fn activate_deactivate_textarea(&mut self) {
        utils::activate(&mut self.textarea[0]);
        utils::deactivate(&mut self.textarea[1]);
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

        // If waiting for a response, render a popup
        if self.state == State::Running {
            self.render_popup(f);
        }

        if let Some(resp) = &self.response {
            let resp = serde_json::to_string_pretty(resp).unwrap();
            let response_paragraph = utils::create_text(&resp, vec![2, 2, 1, 2]);
            f.render_widget(response_paragraph, layout.response_layout[0])
        }
    }

    fn render_popup(&mut self, f: &mut Frame) {
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title("Message")
            .bold()
            .light_green();

        let para = Paragraph::new("Processing...").block(popup_block);
        let area = self.centered_rect(30, 6, f.size());

        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(para, area);
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        // Then cut the middle vertical piece into three width-wise pieces
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1] // Return the middle chunk
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

    fn set_response(&mut self, response: ResultSerde) {
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

    fn render_inputs(&mut self, input: Input) {
        self.textarea[self.which].input(input);
    }
}
