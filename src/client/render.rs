use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

use crate::client::{AppState, Panel, ScrollState};

fn render_panel<'a>(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &Vec<String>,
    scroll: &ScrollState,
    focused: bool,
) {
    let text = items.join("\n");
    let block = if focused {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightYellow))
    } else {
        Block::default().title(title).borders(Borders::ALL)
    };

    let paragraph = Paragraph::new(text).block(block).scroll((scroll.offset, 0));

    f.render_widget(paragraph, area);
}

pub fn ui(f: &mut Frame, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    let container = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    let right_side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(container[1]);

    let stream_height = container[0].height.saturating_sub(2);
    let history_height = right_side[0].height.saturating_sub(2);
    let log_height = right_side[1].height.saturating_sub(2);
    app_state.update_heights(stream_height, history_height, log_height);

    update_scroll(
        &mut app_state.stream_scroll,
        app_state.stream_list.len(),
        stream_height,
    );
    update_scroll(
        &mut app_state.history_scroll,
        app_state.history_list.len(),
        history_height,
    );
    update_scroll(&mut app_state.log_scroll, app_state.logs.len(), log_height);

    render_panel(
        f,
        container[0],
        "Stream",
        &app_state.stream_list,
        &mut app_state.stream_scroll,
        matches!(app_state.focused, Panel::Stream),
    );

    render_panel(
        f,
        right_side[0],
        "Repl History",
        &app_state.history_list,
        &mut app_state.history_scroll,
        matches!(app_state.focused, Panel::History),
    );

    render_panel(
        f,
        right_side[1],
        "Logs",
        &app_state.logs,
        &mut app_state.log_scroll,
        matches!(app_state.focused, Panel::Log),
    );

    let repl = Paragraph::new(Span::styled(
        format!("repl> {}", app_state.repl.clone()),
        Style::default().fg(Color::LightYellow),
    ))
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(repl, chunks[1]);
}

pub fn scroll_up(scroll_state: &mut ScrollState) {
    if scroll_state.offset > 0 {
        scroll_state.offset -= 1;
        scroll_state.follow = false;
    }
}

pub fn scroll_down(scroll_state: &mut ScrollState, total: usize, area_height: u16) {
    let total_u16 = total as u16;
    let max_scroll = total_u16.saturating_sub(area_height);

    if scroll_state.offset < max_scroll {
        scroll_state.offset += 1;
        scroll_state.follow = false;
    } else {
        scroll_state.offset = max_scroll;
        scroll_state.follow = true;
    }
}

fn update_scroll(state: &mut ScrollState, total: usize, area_height: u16) {
    let total_u16 = total as u16;
    if total_u16 <= area_height {
        state.offset = 0;
    } else if state.follow {
        state.offset = total_u16 - area_height;
    }
}
