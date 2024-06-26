use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Padding, Paragraph, Wrap},
};
use tui_textarea::TextArea;

pub fn create_text(text: &str, padding: Vec<u16>) -> Paragraph<'_> {
    Paragraph::new(text)
        .block(Block::new().style(Style::new()).padding(Padding::new(
            padding[0], // left
            padding[1], // right
            padding[2], // top
            padding[3], // bottom
        )))
        .wrap(Wrap { trim: true })
}

pub fn deactivate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());
}

pub fn activate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
}
