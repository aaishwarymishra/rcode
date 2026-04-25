mod render;
mod states;

use crate::states::App;
use crossterm::event;

fn main() -> std::io::Result<()> {
    ratatui::run(run)
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::dummy();
    loop {
        terminal.draw(|frame| render::render(frame, &mut app))?;

        handle_input(&mut app)?;

        if app.exit {
            break Ok(());
        }
    }
}

fn handle_input(app: &mut App) -> std::io::Result<()> {
    match event::read()? {
        event::Event::Key(key) => match key.code {
            event::KeyCode::Down => app.scroll_view_state.scroll_down(),
            event::KeyCode::Up => app.scroll_view_state.scroll_up(),
            event::KeyCode::Enter => {
                let input = app.take_input();
                app.add_message(states::Role::User, input);
            }
            _ => {
                if key.modifiers.contains(event::KeyModifiers::CONTROL)
                    && key.code == event::KeyCode::Char('c')
                {
                    app.exit = true;
                } else {
                    app.text_area.input(key);
                }
            }
        },
        event::Event::Mouse(mouse) => match mouse.kind {
            event::MouseEventKind::ScrollDown => app.scroll_view_state.scroll_down(),
            event::MouseEventKind::ScrollUp => app.scroll_view_state.scroll_up(),
            _ => {}
        },

        _ => {}
    }
    Ok(())
}
