use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, Mutex};

type ClientTx = mpsc::UnboundedSender<String>;
type Clients = Arc<Mutex<HashMap<usize, ClientTx>>>;

static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("127.0.0.1:8888").await.expect("Failed to bind");

    println!("Server listening on port 8888");

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let(socket, addr) = listener.accept().await.expect("Failed to accept");
        println!("New connection from {}", addr);

        let clients = Arc::clone(&clients);

        tokio::spawn(async move {
            handle_client(socket, clients).await;
        });
    }
}

async fn handle_client(socket: TcpStream, clients: Clients) {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    println!("Client {client_id} connected");

    let (reader, mut writer) = socket.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    {
        let mut clients_guard = clients.lock().await;
        clients_guard.insert(client_id, tx);
    }

    let broadcast_message = &format!("Client {client_id} joined the chat").to_string();
    broadcast(&clients, &broadcast_message).await;

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if writer.write_all(message.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    while let Ok(Some(line)) = lines.next_line().await {
        println!("Client {client_id} says: {line}");

        let message = format!("Client {client_id}: {line}");
        broadcast(&clients, &message).await;
    }

    {
        let mut clients_guard = clients.lock().await;
        clients_guard.remove(&client_id);
    }

    let broadcast_message = &format!("Client {client_id} left the chat").to_string();
    broadcast(&clients, &broadcast_message).await;

    println!("Client {client_id} disconnected");
}

async fn broadcast(clients: &Clients, message: &str) {
    let clients_guard = clients.lock().await;

    for tx in clients_guard.values() {
        let _ = tx.send(format!("{message}\n"));
    }
}