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

async fn handle_client(socket: TcpStream, _clients: Clients) {

    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    println!("Client {client_id} connected");

    let (reader, mut writer) = socket.into_split();

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        println!("Received {}", line);

        let response = format!("Echo: {}\n", line);
        if writer.write_all(response.as_bytes()).await.is_err() {
            break;
        }
    }

    println!("Client disconnected");
}