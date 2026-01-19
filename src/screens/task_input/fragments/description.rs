use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{AppState, DraftField, HeuristicsFocus};
use crate::screens::FragmentId;
use crate::screens::common::selected_list_style;
use crate::ui::{dashed_border_set, truncate};

pub fn draw(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let is_active = app.fragment == FragmentId::TaskDescription;
    let title_style = if is_active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let (border_set, border_style) = if is_active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (ratatui::symbols::border::PLAIN, Style::default())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style)
        .title(Span::styled("Task Description [F1]", title_style));
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(32),
            Constraint::Percentage(32),
            Constraint::Percentage(36),
        ])
        .split(inner);

    let name_cursor = draw_name_box(frame, columns[0], app);
    let dataset_cursor = draw_dataset_box(frame, columns[1], app);
    let heuristics_cursor = draw_heuristics_box(frame, columns[2], app);
    name_cursor.or(dataset_cursor).or(heuristics_cursor)
}

fn draw_name_box(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active = app.fragment == FragmentId::TaskDescription && app.draft.field == DraftField::Name;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let value = if app.draft.field == DraftField::Name {
        app.input.clone()
    } else {
        app.draft.name.clone()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Name", title_style));
    frame.render_widget(Paragraph::new(value).block(block), area);
    if active {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1;
        return Some((x, y));
    }
    None
}

fn draw_dataset_box(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active =
        app.fragment == FragmentId::TaskDescription && app.draft.field == DraftField::DatasetFolder;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let value = if app.draft.field == DraftField::DatasetFolder {
        app.input.clone()
    } else {
        app.draft.dataset_folder.clone()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Dataset Folder", title_style));
    frame.render_widget(Paragraph::new(value).block(block), area);
    if active {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1;
        return Some((x, y));
    }
    None
}

fn draw_heuristics_box(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active =
        app.fragment == FragmentId::TaskDescription && app.draft.field == DraftField::Heuristics;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Heuristics [+ to add]", title_style));
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let title_cursor = draw_heuristic_titles(frame, columns[0], app);
    let image_cursor = draw_heuristic_images(frame, columns[1], app);
    title_cursor.or(image_cursor)
}

fn draw_heuristic_titles(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active = app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Titles;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut items = Vec::new();
    if app.draft.heuristics.is_empty() {
        items.push(ListItem::new(Span::styled(
            "[Empty List]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, heuristic) in app.draft.heuristics.iter().enumerate() {
            let selected = idx == app.draft.selected_heuristic;
            let style = if active && selected {
                selected_list_style()
            } else {
                Style::default()
            };
            items.push(ListItem::new(Span::styled(
                truncate(&heuristic.title, 24),
                style,
            )));
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Titles", title_style));
    frame.render_widget(List::new(items).block(block), area);
    if active && !app.draft.heuristics.is_empty() {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1 + app.draft.selected_heuristic as u16;
        return Some((x, y));
    }
    None
}

fn draw_heuristic_images(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active = app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Images;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let images = app
        .draft
        .heuristics
        .get(app.draft.selected_heuristic)
        .map(|h| h.images.as_slice())
        .unwrap_or(&[]);
    let mut items = Vec::new();
    if images.is_empty() {
        items.push(ListItem::new(Span::styled(
            "[Empty List]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, image) in images.iter().enumerate() {
            let selected = idx == app.draft.selected_image;
            let style = if active && selected {
                selected_list_style()
            } else {
                Style::default()
            };
            items.push(ListItem::new(Span::styled(truncate(image, 24), style)));
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Images (Right arrow)", title_style));
    frame.render_widget(List::new(items).block(block), area);
    if active && !images.is_empty() {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1 + app.draft.selected_image as u16;
        return Some((x, y));
    }
    None
}
