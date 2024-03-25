use std::{io, time::Duration};

use arboard::Clipboard;
use ratatui::{
    backend::Backend,
    style::Stylize,
    widgets::{Block, Borders},
    Frame, Terminal,
};
use tui_textarea::{Input, Key, TextArea};

use crate::errors;
use crate::utils;
use crate::MainLayout;

#[derive(Default)]
pub struct App<'a> {
    response: Option<serde_json::Value>,
    textarea: [TextArea<'a>; 2],
    which: usize,
}

impl App<'_> {
    pub fn new() -> Self {
        App::default()
    }

    pub fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        client: &reqwest::Client,
        clipboard: &mut Clipboard,
    ) -> io::Result<()> {
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
                } => return Ok(()),
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
                    let resp = self.request(client, reqwest::Method::GET);
                    self.set_response(resp)
                }
                // POST method
                Input {
                    key: Key::Char('h'),
                    ctrl: true,
                    ..
                } => {
                    let resp = self.request(client, reqwest::Method::POST);
                    self.set_response(resp)
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

    #[tokio::main]
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
