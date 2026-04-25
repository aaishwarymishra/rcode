use crate::states::{App, MessageUtils, Role};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::*;
use tui_scrollview::ScrollView;

pub fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let block = Block::bordered()
        .title("RCODE".cyan().bold().italic())
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .bg(Color::Black);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Max(5),
        ])
        .split(frame.area());

    frame.render_widget(block, frame.area());
    frame.render_widget(
        Paragraph::new("Welcome to Ratatui!").block(Block::new().padding(Padding::left(1))),
        chunks[0],
    );
    frame.render_stateful_widget(
        render_message_history(chunks[1].width, app, true, 1),
        chunks[1],
        &mut app.scroll_view_state,
    );
    frame.render_widget(&app.text_area, chunks[2]);
}

fn render_message_history(width: u16, app: &mut App, has_border: bool, padding: u16) -> ScrollView {
    let total_height = app.get_height(width, has_border);
    let scroll_area = Size::new(width, total_height);

    let mut scroll_view = ScrollView::new(scroll_area)
        .scrollbars_visibility(tui_scrollview::ScrollbarVisibility::Never);

    let mut y = 0;

    for m in &app.messages {
        let height = m.get_height(width, has_border);

        let mut block = Block::default().bg(Color::Black);

        if has_border {
            block = block.border_type(BorderType::Rounded).borders(Borders::ALL);

            match m.role {
                Role::User => {
                    block = block
                        .title("User")
                        .title_alignment(Alignment::Left)
                        .border_style(Color::Green);
                }
                Role::Agent => {
                    block = block
                        .title("Agent")
                        .title_alignment(Alignment::Left)
                        .border_style(Color::Blue);
                }
            }
        }

        let paragraph = Paragraph::new(m.content.clone())
            .block(block.padding(Padding::new(padding, padding, 0, 0)));

        scroll_view.render_widget(paragraph, Rect::new(0, y, width, height));
        y += height;
    }
    scroll_view
}
