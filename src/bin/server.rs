use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("127.0.0.1:8888").await.expect("Failed to bind");

    println!("Server listening on port 8888");

    loop {
        let(socket, addr) = listener.accept().await.expect("Failed to accept");
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}

async fn handle_client(socket: TcpStream) {
    println!("Client connected from {}", socket.peer_addr().unwrap());
}