use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::AppState;
use crate::screens::common::selected_list_style;
use crate::screens::FragmentId;
use crate::ui::truncate;

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect, app: &AppState) {
    let active = app.fragment == FragmentId::TaskHypotheses;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let (border_set, border_style) = if active {
        (crate::ui::dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (ratatui::symbols::border::PLAIN, Style::default())
    };
    let hypothesis_items = app
        .draft
        .hypotheses
        .iter()
        .enumerate()
        .map(|(idx, h)| {
            let selected = idx == app.draft.selected_hypothesis;
            let style = if active && selected {
                selected_list_style()
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(truncate(&h.title, 60), style))
        })
        .collect::<Vec<_>>();

    let hypothesis_list = List::new(hypothesis_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border_set)
            .border_style(border_style)
            .title(Span::styled("Hypotheses [F2]", title_style)),
    );

    frame.render_widget(hypothesis_list, area);
}
