use ratatui::Frame;

use crate::screens::common::draw_header;

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect) {
    draw_header(frame, area);
}
