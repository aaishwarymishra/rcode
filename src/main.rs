mod render;
mod states;
mod theme;

use crate::states::App;
use crate::states::SelectedWidget;
use crossterm::event;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{Chat, Message};
use rig::providers::openai;
use rig::providers::openai::responses_api::ResponsesCompletionModel;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

// A small message enum lets the background worker talk back to the UI thread
// without sharing mutable state across threads.
enum AgentEvent {
    Response(String),
    Error(String),
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // dotenvy reads .env and puts values into process environment variables.
    // After this, `openai::Client::from_env()` can find OPENAI_API_KEY.
    dotenvy::dotenv().ok();
    
    // Enable mouse capture so clicks work
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
    let result = ratatui::run(run);
    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new("gpt-5-mini".to_string());
    let (tx, rx) = mpsc::channel::<AgentEvent>();
    let agent = Arc::new(create_openai_agent(&app));

    loop {
        // Poll the channel first so any finished agent reply shows up before
        // the next frame is drawn.
        drain_agent_events(&mut app, &rx);

        // Draw only the current app state. The UI stays responsive because
        // the network request happens in a background Tokio task.
        terminal.draw(|frame| render::render(frame, &mut app))?;

        // event::poll makes input non-blocking for a short period.
        // That gives us a chance to redraw the screen even when the user is idle.
        if event::poll(Duration::from_millis(30))? {
            handle_input(&mut app, &tx, agent.clone())?;
        }

        drain_agent_events(&mut app, &rx);

        if app.exit {
            break Ok(());
        }
    }
}

fn handle_input(
    app: &mut App,
    tx: &Sender<AgentEvent>,
    agent: Arc<rig::agent::Agent<ResponsesCompletionModel>>,
) -> std::io::Result<()> {
    match event::read()? {
        event::Event::Key(key) => match key.code {
            event::KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                app.exit = true;
            }
            event::KeyCode::Enter if matches!(app.selected_widget, SelectedWidget::TextArea) => {
                let input = app.take_input();
                // Ignore empty prompts and prevent overlapping requests.
                if input.trim().is_empty() || app.is_generating {
                    return Ok(());
                }

                app.add_message(states::Role::User, input.clone());
                app.scroll_view_state.scroll_to_bottom();
                app.is_generating = true;
                app.status = Some("Thinking...".to_string());

                // Start the request in the background so the UI thread never waits on the API.
                let message_history = app.message_history();
                spawn_agent_request(input, tx.clone(), agent, message_history);
            }
            event::KeyCode::Up if matches!(app.selected_widget, SelectedWidget::MessageHistory) => {
                app.scroll_view_state.scroll_up();
            }
            event::KeyCode::Down if matches!(app.selected_widget, SelectedWidget::MessageHistory) => {
                app.scroll_view_state.scroll_down();
            }
            event::KeyCode::PageUp if matches!(app.selected_widget, SelectedWidget::MessageHistory) => {
                app.scroll_view_state.scroll_page_up();
            }
            event::KeyCode::PageDown if matches!(app.selected_widget, SelectedWidget::MessageHistory) => {
                app.scroll_view_state.scroll_page_down();
            }
            event::KeyCode::Home if matches!(app.selected_widget, SelectedWidget::MessageHistory) => {
                app.scroll_view_state.scroll_to_top();
            }
            event::KeyCode::End if matches!(app.selected_widget, SelectedWidget::MessageHistory) => {
                app.scroll_view_state.scroll_to_bottom();
            }
            _ => {
                if matches!(app.selected_widget, SelectedWidget::TextArea) {
                    app.text_area.input(key);
                }
            }
        },
        event::Event::Mouse(mouse) => {
            if let event::MouseEventKind::Down(event::MouseButton::Left) = mouse.kind {
                app.set_selected_widget_from_position(mouse.column, mouse.row);
            }

            // Always allow scrolling the scrollview with the mouse wheel regardless of focus
            match mouse.kind {
                event::MouseEventKind::ScrollDown => app.scroll_view_state.scroll_down(),
                event::MouseEventKind::ScrollUp => app.scroll_view_state.scroll_up(),
                _ => {}
            }
        }

        _ => {}
    }
    Ok(())
}

fn drain_agent_events(app: &mut App, rx: &Receiver<AgentEvent>) {
    while let Ok(event) = rx.try_recv() {
        match event {
            AgentEvent::Response(response) => {
                app.add_message(states::Role::Agent, response);
                app.is_generating = false;
                app.status = None;
                app.scroll_view_state.scroll_to_bottom();
            }
            AgentEvent::Error(error) => {
                app.status = Some(format!("Agent error: {error}"));
                app.is_generating = false;
            }
        }
    }
}

fn create_openai_agent(app: &App) -> rig::agent::Agent<ResponsesCompletionModel> {
    openai::Client::from_env()
        .agent(app.model.as_deref().expect("Model not set"))
        .preamble("You are a helpful coding assistant inside a terminal UI. Keep responses concise and useful.")
        .build()
}

fn spawn_agent_request(
    prompt: String,
    tx: Sender<AgentEvent>,
    agent: Arc<rig::agent::Agent<ResponsesCompletionModel>>,
    message_history: Vec<Message>,
) {
    tokio::spawn(async move {
        let result = agent
            .chat(Message::user(prompt), message_history)
            .await
            .map_err(|error| error.to_string());

        let _ = match result {
            Ok(response) => tx.send(AgentEvent::Response(response)),
            Err(error) => tx.send(AgentEvent::Error(error)),
        };
    });
}
