use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    Frame,
};

use std::rc::Rc;

pub struct MainLayout {
    pub main_layout: Rc<[Rect]>,
    pub request_layout: Rc<[Rect]>,
    pub response_layout: Rc<[Rect]>,
}

impl MainLayout {
    pub fn new(f: &mut Frame) -> Self {
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
        MainLayout {
            main_layout,
            request_layout,
            response_layout,
        }
    }
}
