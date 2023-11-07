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
use std::{
    io::{stdout, Result},
};

struct App {
    request: Option<serde_json::Value>,
    response: Option<serde_json::Value>,
}

impl App {
    fn new(request: Option<serde_json::Value>, response: Option<serde_json::Value>) -> Self {
        Self { request, response }
    }

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
                .title("Request Url"),
            request_layout[0],
        );
        f.render_widget(
            Block::default()
                .borders(Borders::all())
                .blue()
                .title("Json"),
            request_layout[1],
        );
        f.render_widget(
            Block::default()
                .borders(Borders::all())
                .green()
                .title("Response"),
            response_layout[0],
        );

        if self.response.is_some() {
            let r = create_text("Got a response, add response here", vec![2, 2, 1, 2]);
            f.render_widget(r, response_layout[0])
        }
    }

    #[tokio::main]
    async fn test_get_client(&mut self) -> std::result::Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://httpbin.org/get")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        self.store_response(Ok(&resp));
        Ok(())
    }

    fn store_response(&mut self, response: std::result::Result<&serde_json::Value, reqwest::Error>) {
        if let Ok(response) = response {
            self.response = Some(response.clone())
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

    let mut app = App::new(None, None);

    loop {
        terminal.draw(|f| app.render_ui(f))?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
                else if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('t') {
                    app.test_get_client();
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
