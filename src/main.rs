use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};
use std::{
    io::{stdout, Result},
    rc::Rc,
};

struct App<'a> {
    request: Option<&'a str>,
    response: Option<&'a str>,
}

impl<'a> App<'a> {
    fn new(request: Option<&'a str>, response: Option<&'a str>) -> Self {
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
    }

    #[tokio::main]
    async fn test_get_client(&self) -> std::result::Result<std::string::String, reqwest::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://httpbin.org/get")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        Ok(resp.to_string())
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

    let app = App::new(None, None);

    loop {
        terminal.draw(|f| app.render_ui(f))?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
