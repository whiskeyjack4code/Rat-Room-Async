use tokio::net::TcpStream;

const SOCKET: &str = "127.0.0.1:8888";
#[tokio::main]
async fn main() {
    let _stream = TcpStream::connect(SOCKET).await.expect("Failed to connect");

    println!("Connected to {}", SOCKET);
}