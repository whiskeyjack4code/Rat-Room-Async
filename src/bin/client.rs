use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const SOCKET: &str = "127.0.0.1:8888";
#[tokio::main]
async fn main() {
    let stream = TcpStream::connect(SOCKET).await.expect("Failed to connect");
    println!("Connected to {}", SOCKET);

    let (reader, mut writer) = stream.into_split();

    tokio::spawn(async move {
        let mut server_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();

            match server_reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => print!("{}", line),
                Err(_) => break,
            }
        }
    });

    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);
    let mut input = String::new();

    loop {
        input.clear();
        stdin_reader.read_line(&mut input).await.unwrap();
        writer.write_all(input.as_bytes()).await.unwrap();
    }

}