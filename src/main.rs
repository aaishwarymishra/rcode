mod render;
mod states;
mod theme;

use crate::states::App;
use crossterm::event;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::openai;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

// A small message enum lets the background worker talk back to the UI thread
// without sharing mutable state across threads.
enum AgentEvent {
    Response(String),
    Error(String),
}

fn main() -> std::io::Result<()> {
    // dotenvy reads .env and puts values into process environment variables.
    // After this, `openai::Client::from_env()` can find OPENAI_API_KEY.
    dotenvy::dotenv().ok();
    ratatui::run(run)
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::dummy();
    let (tx, rx) = mpsc::channel::<AgentEvent>();

    loop {
        // Poll the channel first so any finished agent reply shows up before
        // the next frame is drawn.
        drain_agent_events(&mut app, &rx);

        // Draw only the current app state. The UI stays responsive because
        // the network request happens on another thread.
        terminal.draw(|frame| render::render(frame, &mut app))?;

        // event::poll makes input non-blocking for a short period.
        // That gives us a chance to redraw the screen even when the user is idle.
        if event::poll(Duration::from_millis(30))? {
            handle_input(&mut app, &tx)?;
        }

        drain_agent_events(&mut app, &rx);

        if app.exit {
            break Ok(());
        }
    }
}

fn handle_input(app: &mut App, tx: &Sender<AgentEvent>) -> std::io::Result<()> {
    match event::read()? {
        event::Event::Key(key) => match key.code {
            event::KeyCode::Down => app.scroll_view_state.scroll_down(),
            event::KeyCode::Up => app.scroll_view_state.scroll_up(),
            event::KeyCode::Enter => {
                let input = app.take_input();
                // Ignore empty prompts and prevent overlapping requests.
                if input.trim().is_empty() || app.is_generating {
                    return Ok(());
                }

                app.add_message(states::Role::User, input.clone());
                app.is_generating = true;
                app.status = Some("Thinking...".to_string());

                // Spawn a worker thread so the UI thread never waits on the API.
                spawn_agent_request(input, tx.clone());
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

fn drain_agent_events(app: &mut App, rx: &Receiver<AgentEvent>) {
    while let Ok(event) = rx.try_recv() {
        match event {
            AgentEvent::Response(response) => {
                app.add_message(states::Role::Agent, response);
                app.is_generating = false;
                app.status = None;
            }
            AgentEvent::Error(error) => {
                app.status = Some(format!("Agent error: {error}"));
                app.is_generating = false;
            }
        }
    }
}

fn spawn_agent_request(prompt: String, tx: Sender<AgentEvent>) {
    thread::spawn(move || {
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())
            .and_then(|rt| {
                rt.block_on(async move {
                    // This client reads OPENAI_API_KEY from the environment.
                    // We loaded .env in main(), so the key can live in a local file
                    // instead of being hardcoded in the source.
                    let client = openai::Client::from_env();
                    let agent = client
                        .agent("gpt-5-mini")
                        .preamble("You are a helpful coding assistant inside a terminal UI. Keep responses concise and useful.")
                        .build();

                    agent.prompt(&prompt).await.map_err(|error| error.to_string())
                })
            });

        let _ = match result {
            Ok(response) => tx.send(AgentEvent::Response(response)),
            Err(error) => tx.send(AgentEvent::Error(error.to_string())),
        };
    });
}
