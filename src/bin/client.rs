#[path = "../protocol.rs"]
mod protocol;

use protocol::{ClientMessage, ServerMessage};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{tcp::OwnedWriteHalf, TcpStream};

const SOCKET: &str = "127.0.0.1:8888";

async fn send_json(
    writer: &mut OwnedWriteHalf,
    message: &ClientMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string(message)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    let mut username = String::new();
    println!("Enter username:");
    stdin_reader.read_line(&mut username).await.unwrap();

    let username = username.trim().to_string();

    let stream = TcpStream::connect(SOCKET)
        .await
        .expect("Failed to connect");

    println!("Connected to {SOCKET}");
    println!("\nCommands:");
    println!("/join <room>");
    println!("/leave");
    println!("/rooms\n");
    println!("/help (Prints this menu again)\n");

    let (reader, mut writer) = stream.into_split();

    let _ = send_json(
        &mut writer,
        &ClientMessage::SetUsername {
            username: username.clone(),
        },
    )
        .await;

    tokio::spawn(async move {
        let mut server_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();

            match server_reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("\nServer closed the connection.");
                    break;
                }
                Ok(_) => {
                    let parsed: Result<ServerMessage, _> = serde_json::from_str(line.trim());

                    match parsed {
                        Ok(ServerMessage::Welcome { message }) => {
                            println!("[welcome] {message}");
                        }
                        Ok(ServerMessage::Error { message }) => {
                            println!("[error] {message}");
                        }
                        Ok(ServerMessage::System { message }) => {
                            println!("[system] {message}");
                        }
                        Ok(ServerMessage::RoomJoined { room }) => {
                            println!("[room] Joined {room}");
                        }
                        Ok(ServerMessage::Chat {
                               username,
                               room,
                               message,
                           }) => {
                            println!("[{room}] {username}: {message}");
                        }
                        Ok(ServerMessage::RoomList { rooms }) => {
                            println!("[rooms] {}", rooms.join(", "));
                        }
                        Err(_) => {
                            println!("[raw] {}", line.trim());
                        }
                    }
                }
                Err(_) => {
                    println!("\nLost connection to server.");
                    break;
                }
            }
        }
    });

    let mut input = String::new();

    loop {
        input.clear();

        let bytes_read = stdin_reader.read_line(&mut input).await.unwrap();
        if bytes_read == 0 {
            break;
        }

        let message = input.trim();
        if message.is_empty() {
            continue;
        }

        if message == "/leave" {
            let _ = send_json(&mut writer, &ClientMessage::LeaveRoom).await;
            continue;
        }

        if message == "/rooms" {
            let _ = send_json(&mut writer, &ClientMessage::ListRooms).await;
            continue;
        }

        if message == "/help" {
            println!("\nCommands:");
            println!("/join <room>");
            println!("/leave");
            println!("/rooms\n");
            continue;
        }

        if let Some(room_name) = message.strip_prefix("/join ") {
            let room_name = room_name.trim();

            if room_name.is_empty() {
                println!("Usage: /join room-name");
                continue;
            }

            let _ = send_json(
                &mut writer,
                &ClientMessage::JoinRoom {
                    room: room_name.to_string(),
                },
            )
                .await;

            continue;
        }

        let _ = send_json(
            &mut writer,
            &ClientMessage::Chat {
                message: message.to_string(),
            },
        )
            .await;
    }
}