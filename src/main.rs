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

fn create_text<'a>(text: &str, padding: Vec<u16>) -> Paragraph<'_> {
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

fn ui(f: &mut Frame) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());
    let request_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(7), Constraint::Percentage(93)])
        .split(main_layout[0]);
    let response_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(main_layout[1]);

    let p = create_text("Json for request here (If needed)", vec![2, 2, 2, 2]);
    let r = create_text("{Something: test}", vec![2, 2, 1, 2]);
    let url = create_text("https://test.com/api", vec![2, 1, 1, 1]);
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

    f.render_widget(url, request_layout[0]);
    f.render_widget(p, request_layout[1]);
    f.render_widget(r, response_layout[0])
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(ui)?;
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
