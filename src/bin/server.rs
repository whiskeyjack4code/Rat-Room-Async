use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, Mutex};

#[derive(Clone)]
struct Client{
    username: String,
    tx: mpsc::UnboundedSender<String>,
}

type ClientTx = mpsc::UnboundedSender<String>;
type Clients = Arc<Mutex<HashMap<usize, Client>>>;

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

    let (reader, mut writer) = socket.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let username = match lines.next_line().await {
        Ok(Some(name)) => name.trim().to_string(),
        _ => return,
    };

    println!("Client {client_id} wants username: {username}");
}

async fn broadcast(clients: &Clients, message: &str) {
    let clients_guard = clients.lock().await;

    for tx in clients_guard.values() {
        let _ = tx.send(format!("{message}\n"));
    }
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