use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};
use ratatui_textarea::TextArea;

use crate::MainLayout;

fn create_text(text: &str, padding: Vec<u16>) -> Paragraph<'_> {
    Paragraph::new(text)
        .block(
            Block::new()
                .style(Style::new().bg(Color::Black))
                .padding(Padding::new(
                    padding[0], // left
                    padding[1], // right
                    padding[2], // top
                    padding[3], // bottom
                )),
        )
        .wrap(Wrap { trim: true })
}

pub struct App<'a> {
    request: String,
    body_json: String,
    response: Option<serde_json::Value>,
    textarea: [TextArea<'a>; 2],
}

impl Default for App<'_> {
    fn default() -> Self {
        App {
            request: String::new(),
            body_json: String::new(),
            response: None,
            textarea: [TextArea::default(), TextArea::default()]
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
            let r = create_text(&resp, vec![2, 2, 1, 2]);
            f.render_widget(r, layout.response_layout[0])
        }
    }

    #[tokio::main]
    pub async fn get_response(
        &mut self,
        client: &reqwest::Client,
    ) -> std::result::Result<serde_json::Value, reqwest::Error> {
        let resp = client
            .get("https://httpbin.org/get") // TODO: Add here the request url that comes from the request text url (Not yet implemented)
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
}
