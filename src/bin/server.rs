#[path = "../protocol.rs"]
mod protocol;

use protocol::{ClientMessage, ServerMessage};

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use serde_json;

#[derive(Clone)]
struct Client {
    username: String,
    tx: mpsc::UnboundedSender<String>,
}

type Clients = Arc<Mutex<HashMap<usize, Client>>>;

static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8888")
        .await
        .expect("Failed to bind");

    println!("Server listening on 127.0.0.1:8888");

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, addr) = listener.accept().await.expect("Failed to accept");
        println!("New connection from {addr}");

        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            handle_client(socket, clients).await;
        });
    }
}

async fn send_json(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    message: &ServerMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string(&message)?;

    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    Ok(())
}

fn is_valid_username(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 20
}

async fn username_exists(clients: &Clients, username: &str) -> bool {
    let clients_guard = clients.lock().await;

    clients_guard
        .values()
        .any(|client| client.username.eq_ignore_ascii_case(username))
}

async fn broadcast(clients: &Clients, message: &str) {
    let clients_guard = clients.lock().await;

    for client in clients_guard.values() {
        let _ = client.tx.send(format!("{message}\n"));
    }
}

async fn handle_client(socket: TcpStream, clients: Clients) {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    let (reader, mut writer) = socket.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let username = match lines.next_line().await {
        Ok(Some(name)) => name.trim().to_string(),
        _ => return,
    };

    if !is_valid_username(&username) {
        let _ = writer.write_all(b"Invalid username.\n").await;
        return;
    }

    if username_exists(&clients, &username).await {
        let _ = writer.write_all(b"Username already taken.\n").await;
        return;
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    {
        let mut clients_guard = clients.lock().await;
        clients_guard.insert(
            client_id,
            Client {
                username: username.clone(),
                tx,
            },
        );
    }

    let welcome = format!("Welcome, {username}!\n");
    let _ = writer.write_all(welcome.as_bytes()).await;

    println!("Client {client_id} registered as {username}");

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if writer.write_all(message.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    broadcast(&clients, &format!("{username} joined the chat")).await;

    while let Ok(Some(line)) = lines.next_line().await {
        let message = line.trim();

        if message.is_empty() {
            continue;
        }

        println!("{username}: {message}");
        broadcast(&clients, &format!("{username}: {message}")).await;
    }

    {
        let mut clients_guard = clients.lock().await;
        clients_guard.remove(&client_id);
    }

    broadcast(&clients, &format!("{username} left the chat")).await;

    println!("{username} disconnected");
}