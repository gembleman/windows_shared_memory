# Windows Shared Memory

Windows IPC library using shared memory. Supports 32-bit/64-bit cross-process communication.

## Installation

```toml
[dependencies]
windows_shared_memory = "0.1.4"
```

## Usage

```rust
use windows_shared_memory::{Server, Client, ReceiveMessage};

// Server
let server = Server::new(None).unwrap();
server.send(b"Hello").unwrap();
if let ReceiveMessage::Message(msg) = server.receive(Some(1000)) {
    println!("{}", msg);
}

// Client (separate process)
let client = Client::new(None).unwrap();
if let ReceiveMessage::Message(msg) = client.receive(Some(1000)) {
    println!("{}", msg);
}
client.send(b"Hi").unwrap();
```

Custom path:
```rust
let server = Server::new(Some("Local\\MyShm")).unwrap();
let client = Client::new(Some("Local\\MyShm")).unwrap();
```

## API

| Method                | Description          |
| --------------------- | -------------------- |
| `Server::new(path)`   | Create server        |
| `Client::new(path)`   | Connect to server    |
| `send(&[u8])`         | Send data (max 16KB) |
| `receive(timeout_ms)` | Receive data         |
| `server.send_close()` | Close signal         |

`ReceiveMessage`: `Message(String)`, `Timeout`, `Exit`, `MessageError(String)`

## License

MIT
