# Async Rust Chat Server (Tokio + TUI)

A real-time, multi-client chat application built in Rust using Tokio for async networking and Ratatui for a terminal-based user interface.

The goal of this project is to build a complete chat system from first principles. It avoids frameworks and focuses on understanding how async networking, message routing, and shared state work under the hood.

---

## Features

- Async TCP server using Tokio  
- Multiple concurrent client connections  
- Username system with validation  
- Room-based chat with a default lobby  
- Join, leave, and list rooms  
- JSON-based message protocol  
- Broadcast system using channels  
- Terminal UI built with Ratatui  
- Scrollable message view  
- Input box with live typing  
- Message buffer to prevent unbounded growth  

---

## Getting Started

### Clone the repository

```bash
git clone https://github.com/whiskeyjack4code/Rat-Room-Async
cd Rat-Room-Async
```

---

## Configuration

This project uses two separate configuration files:

```text
server.toml   → controls how the server listens
client.toml   → controls where the client connects
```

### Default values

#### `server.toml`

```toml
host = "0.0.0.0"
port = 8888
```

#### `client.toml`

```toml
host = "127.0.0.1"
port = 8888
```

These defaults allow the application to run locally without any changes.

---

### Running locally

With the default configuration:

```bash
cargo run --bin server
```

In another terminal:

```bash
cargo run --bin client
```

Open multiple clients to simulate multiple users.

---

### Running against a remote server (e.g. AWS)

If you deploy the server to a remote machine:

1. Update `client.toml`:

```toml
host = "YOUR_SERVER_PUBLIC_IP"
port = 8888
```

2. Run the client locally as usual:

```bash
cargo run --bin client
```

---

### Important note on server configuration

The server should use:

```toml
host = "0.0.0.0"
```

This allows it to accept connections from outside the machine.

Using `127.0.0.1` will restrict access to the local machine only.

---

## Commands

The client supports the following commands:

```
/join <room>   Join or create a room
/leave         Return to the lobby
/rooms         List active rooms
Esc            Exit the application
```

---

## Project Structure

```
src/
├── bin/
│   ├── server.rs
│   └── client.rs
├── protocol.rs
```

- `server.rs` handles incoming connections, manages clients, and routes messages between rooms  
- `client.rs` contains the terminal UI, input handling, and rendering loop  
- `protocol.rs` defines the shared message types used between client and server  

---

## How It Works

Clients connect to the server over TCP and send an initial message to set their username.

The server maintains a shared list of connected clients using a synchronized data structure. Each client has its own message channel, allowing the server to broadcast messages asynchronously without blocking.

Messages are routed based on the client's current room. When a message is received, it is only sent to other clients in the same room.

On the client side, a single event loop handles:
- incoming messages from the server
- keyboard input
- UI rendering

This allows the interface to remain responsive while handling network activity.

---

## Running on a Remote Server

The server can be deployed to a remote machine (for example, an Ubuntu instance on AWS).

Basic steps:
- run the server binary on the remote machine  
- ensure port `8888` is open  
- update `client.toml` with the server’s public IP  

---

## Future Improvements

The following features were intentionally left out to keep the project focused:

- Message persistence (database)  
- User authentication  
- Logging and observability  
- Containerization (Docker)  
- TLS encryption  

These can be added on top of the existing architecture if needed.

---

## Purpose

This project is intended as a practical introduction to:
- async programming in Rust  
- networked application design  
- message-driven systems  
- building real applications without relying on frameworks  

---

## License

MIT License
