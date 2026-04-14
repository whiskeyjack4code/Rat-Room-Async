use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const SOCKET: &str = "127.0.0.1:8888";
#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect(SOCKET).await.expect("Failed to connect");
    println!("Connected to {}", SOCKET);

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();

    loop {
        input.clear();
        reader.read_line(&mut input).await.unwrap();
        stream.write_all(input.trim_end().as_bytes()).await.unwrap();
    }

}