use tokio::net::TcpListener;

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("127.0.0.1:8888").await.expect("Failed to bind");

    println!("Server listening on port 8888");

    loop {
        let(_socket, addr) = listener.accept().await.expect("Failed to accept");
        println!("New connection from {}", addr);
    }

}