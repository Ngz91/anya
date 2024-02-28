use ratatui::{
    style::{Stylize},
    widgets::{Block, Borders},
    Frame,
};
use ratatui_textarea::{TextArea, Input};

use crate::MainLayout;
use crate::utils;

pub struct App<'a> {
    request: String,
    body_json: String,
    response: Option<serde_json::Value>,
    textarea: [TextArea<'a>; 2],
    which: usize,
}

impl Default for App<'_> {
    fn default() -> Self {
        App {
            request: String::new(),
            body_json: String::new(),
            response: None,
            textarea: [TextArea::default(), TextArea::default()],
            which: 0
        }
    }
}

impl App<'_> {
    pub fn render_ui(&mut self, f: &mut Frame, layout: &MainLayout) {
        self.textarea[0].set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Request Url")
                .bold()
                .blue(),
        );
        self.textarea[1].set_block(
            Block::default()
                .borders(Borders::all())
                .blue()
                .title("Json")
                .bold(),
        );

        f.render_widget(self.textarea[0].widget(), layout.request_layout[0]);
        f.render_widget(self.textarea[1].widget(), layout.request_layout[1]);
        f.render_widget(
            Block::default()
                .borders(Borders::all())
                .green()
                .title("Response")
                .bold(),
            layout.response_layout[0],
        );

        if let Some(resp) = &self.response {
            let resp = serde_json::to_string_pretty(resp).unwrap();
            let r = utils::create_text(&resp, vec![2, 2, 1, 2]);
            f.render_widget(r, layout.response_layout[0])
        }
    }

    #[tokio::main]
    pub async fn get_response(
        &mut self,
        client: &reqwest::Client,
    ) -> std::result::Result<serde_json::Value, reqwest::Error> {
        let request_url = &self.textarea[0].lines()[0]; // http://httpbin.org/get for tests
        let _request_json = &self.textarea[1].lines()[0]; // TODO get json and serialize
        let resp = client
            .get(request_url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        Ok(resp)
    }

    pub fn set_response(
        &mut self,
        response: std::result::Result<serde_json::Value, reqwest::Error>,
    ) {
        if let Ok(resp) = response {
            self.response = Some(resp)
        }
    }

    pub fn handle_textarea_events(&mut self) {
        utils::inactivate(&mut self.textarea[self.which]);
        self.which = (self.which + 1) % 2;
        utils::activate(&mut self.textarea[self.which])
    }

    pub fn handle_inputs(&mut self, input: Input) {
        self.textarea[self.which].input(input);
    }

    pub fn activate_deactivate_textarea(&mut self) {
        utils::activate(&mut self.textarea[0]);
        utils::inactivate(&mut self.textarea[1]);
    }
}
