use crate::states::{App, MessageUtils, Role};
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use tui_scrollview::ScrollView;

pub fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let block = Block::bordered()
        .title("RCODE")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::BORDER)
        .title_style(Style::default().fg(theme::BORDER_LIGHT).bold().italic())
        .style(Style::default().bg(theme::BACKGROUND).fg(theme::TEXT));

    let inner = block.inner(frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Max(5),
        ])
        .split(inner);

    frame.render_widget(block, frame.area());
    frame.render_widget(
        Paragraph::new("Rust-powered CLI coding agent")
            .style(theme::MUTED)
            .block(Block::new().padding(Padding::left(1))),
        chunks[0],
    );
    let scroll_view = render_message_history(chunks[1].width, app, true, 1);
    frame.render_stateful_widget(scroll_view, chunks[1], &mut app.scroll_view_state);
    frame.render_widget(&app.text_area, chunks[2]);
}

fn render_message_history(width: u16, app: &mut App, has_border: bool, padding: u16) -> ScrollView {
    let total_height = app.get_height(width, has_border);
    let scroll_area = Size::new(width, total_height);

    let mut scroll_view = ScrollView::new(scroll_area)
        .scrollbars_visibility(tui_scrollview::ScrollbarVisibility::Never);

    let mut y = 0;

    let mut block = Block::default().style(Style::default().bg(theme::SURFACE).fg(theme::TEXT));

    if has_border {
        let (label, accent) = match m.role {
            Role::User => ("User", theme::USER),
            Role::Agent => ("Agent", theme::AGENT),
        };

        block = block
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(accent)
            .title(label)
            .title_alignment(Alignment::Left)
            .title_style(Style::default().fg(accent).bold());
    }

    for m in &mut app.messages {
        let height = m.get_height(width, has_border);

        let paragraph = Paragraph::new(m.content.as_str())
            .block(block.clone().padding(Padding::new(padding, padding, 0, 0)));

        scroll_view.render_widget(paragraph, Rect::new(0, y, width, height));
        y += height;
    }
    scroll_view
}
