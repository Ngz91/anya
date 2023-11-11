use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Terminal},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};
use std::io::{stdout, Result};

#[derive(Default)]
struct App {
    request: Option<serde_json::Value>,
    response: Option<serde_json::Value>,
}

impl App {
    fn render_ui(&self, f: &mut Frame) {
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

        f.render_widget(
            Block::default()
                .borders(Borders::all())
                .blue()
                .title("Request Url")
                .bold(),
            request_layout[0],
        );
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
    async fn get_response(
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

    fn set_response(&mut self, response: std::result::Result<serde_json::Value, reqwest::Error>) {
        if let Ok(resp) = response {
            self.response = Some(resp)
        }
    }
}

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

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = App::default();
    let client = reqwest::Client::new();

    loop {
        terminal.draw(|f| app.render_ui(f))?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                } else if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('g') {
                    let resp = app.get_response(&client);
                    app.set_response(resp)
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
