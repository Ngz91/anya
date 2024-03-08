use ratatui::{
    style::Stylize,
    widgets::{Block, Borders},
    Frame,
};
use ratatui_textarea::{Input, TextArea};

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

    pub fn render_ui(&mut self, f: &mut Frame, layout: &MainLayout) {
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

    #[tokio::main]
    pub async fn request(
        &mut self,
        client: &reqwest::Client,
        method: reqwest::Method,
    ) -> std::result::Result<serde_json::Value, errors::CustomError> {
        let request_url = &self.textarea[0].lines()[0];

        let has_json = !self.textarea[1].lines()[0].is_empty();

        let mut request_builder = client.request(method, request_url);

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

    pub fn set_response(
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

    pub fn change_textarea(&mut self) {
        utils::deactivate(&mut self.textarea[self.which]);
        self.which = (self.which + 1) % 2;
        utils::activate(&mut self.textarea[self.which])
    }

    pub fn handle_inputs(&mut self, input: Input) {
        self.textarea[self.which].input(input);
    }

    pub fn activate_deactivate_textarea(&mut self) {
        utils::activate(&mut self.textarea[0]);
        utils::deactivate(&mut self.textarea[1]);
    }
}
