use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};
use ratatui_textarea::TextArea;

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

enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    input_mode: InputMode,
    request: String,
    body_json: String,
    response: Option<serde_json::Value>,
}

impl Default for App {
    fn default() -> Self {
        App {
            input_mode: InputMode::Normal,
            request: String::new(),
            body_json: String::new(),
            response: None,
        }
    }
}

impl App {
    pub fn render_ui(&self, f: &mut Frame) {
        let mut request_textarea = TextArea::default();
        request_textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Request Url")
                .bold()
                .blue(),
        );

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.size());
        let request_layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .constraints([Constraint::Percentage(7), Constraint::Percentage(93)])
            .split(main_layout[0]);
        let response_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(main_layout[1]);

        f.render_widget(request_textarea.widget(), request_layout[0]);
        f.render_widget(
            Block::default()
                .borders(Borders::all())
                .blue()
                .title("Json")
                .bold(),
            request_layout[1],
        );
        f.render_widget(
            Block::default()
                .borders(Borders::all())
                .green()
                .title("Response")
                .bold(),
            response_layout[0],
        );

        if let Some(resp) = &self.response {
            let resp = serde_json::to_string_pretty(resp).unwrap();
            let r = create_text(&resp, vec![2, 2, 1, 2]);
            f.render_widget(r, response_layout[0])
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