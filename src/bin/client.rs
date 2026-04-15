#[path = "../protocol.rs"]
mod protocol;

use protocol::{ClientMessage, ServerMessage};

use crossterm::{
    event::{self, Event, KeyEventKind, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{tcp::OwnedWriteHalf, TcpStream};
use tokio::sync::mpsc;

const SOCKET: &str = "127.0.0.1:8888";

struct App {
    messages: Vec<String>,
    input: String,
    username: String,
    room: String,
}

async fn send_json(
    writer: &mut OwnedWriteHalf,
    message: &ClientMessage,
) {
    if let Ok(json) = serde_json::to_string(message) {
        let _ = writer.write_all(json.as_bytes()).await;
        let _ = writer.write_all(b"\n").await;
    }
}

async fn handle_input(app: &mut App, writer: &mut OwnedWriteHalf) {
    let message = app.input.trim().to_string();

    if message.is_empty() {
        return;
    }

    if message == "/leave" {
        send_json(writer, &ClientMessage::LeaveRoom).await;
    } else if message == "/rooms" {
        send_json(writer, &ClientMessage::ListRooms).await;
    } else if let Some(room) = message.strip_prefix("/join ") {
        send_json(
            writer,
            &ClientMessage::JoinRoom {
                room: room.trim().to_string(),
            },
        )
            .await;
    } else {
        send_json(
            writer,
            &ClientMessage::Chat {
                message: message.clone(),
            },
        )
            .await;
    }

    app.input.clear();
}

fn draw_ui(frame: &mut ratatui::Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let messages = Paragraph::new(app.messages.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Chat"));

    frame.render_widget(messages, layout[0]);

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, layout[1]);

    let status = Paragraph::new(format!(
        "User: {} | Room: {}",
        app.username, app.room
    ));

    frame.render_widget(status, layout[2]);
}

#[tokio::main]
async fn main() {
    // Username input BEFORE raw mode
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut username = String::new();

    println!("Enter username:");
    stdin.read_line(&mut username).await.unwrap();

    let username = username.trim().to_string();

    let stream = TcpStream::connect(SOCKET)
        .await
        .expect("Failed to connect");

    let (reader, mut writer) = stream.into_split();

    send_json(
        &mut writer,
        &ClientMessage::SetUsername {
            username: username.clone(),
        },
    )
        .await;

    // Setup message channel
    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();

            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<ServerMessage>(line.trim()) {
                        let _ = tx.send(msg);
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Setup TUI
    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App {
        messages: Vec::new(),
        input: String::new(),
        username,
        room: "lobby".to_string(),
    };

    // Main UI loop
    loop {
        // Handle incoming server messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                ServerMessage::Welcome { message } => {
                    app.messages.push(format!("[welcome] {message}"));
                }
                ServerMessage::System { message } => {
                    app.messages.push(format!("[system] {message}"));
                }
                ServerMessage::Chat { username, room, message } => {
                    app.messages.push(format!("[{room}] {username}: {message}"));
                }
                ServerMessage::RoomJoined { room } => {
                    app.room = room.clone();
                    app.messages.push(format!("[room] Joined {room}"));
                }
                ServerMessage::RoomList { rooms } => {
                    app.messages.push(format!("[rooms] {}", rooms.join(", ")));
                }
                ServerMessage::Error { message } => {
                    app.messages.push(format!("[error] {message}"));
                }
            }
        }

        terminal.draw(|f| draw_ui(f, &app)).unwrap();

        // Handle keyboard input
        if event::poll(std::time::Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            handle_input(&mut app, &mut writer).await;
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}