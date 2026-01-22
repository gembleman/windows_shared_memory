# Windows Shared Memory

Windows IPC library using shared memory. Supports 32-bit/64-bit cross-process communication.



## Installation

```toml
[dependencies]
windows_shared_memory = "0.1.5"
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

## Custom Buffer Size

```rust
// Server with 64KB buffer
let server = Server::with_buffer_size(None, 64 * 1024).unwrap();

// Client auto-detects buffer size from server
let client = Client::new(None).unwrap();
println!("Buffer: {} KB", client.buffer_size() / 1024);
```

## API

| Method                                 | Description               |
| -------------------------------------- | ------------------------- |
| `Server::new(path)`                    | Create server (16KB)      |
| `Server::with_buffer_size(path, size)` | Create with custom buffer |
| `Client::new(path)`                    | Connect to server         |
| `send(&[u8])`                          | Send data                 |
| `receive(timeout_ms)`                  | Receive as String         |
| `receive_bytes(timeout_ms)`            | Receive as bytes          |
| `buffer_size()`                        | Get buffer size           |
| `server.send_close()`                  | Close signal              |

`ReceiveMessage`: `Message(String)`, `Timeout`, `Exit`, `MessageError(String)`

## vs winmmf

|                  | windows_shared_memory            | winmmf                 |
| ---------------- | -------------------------------- | ---------------------- |
| **Design**       | Server-Client IPC                | Generic MMF wrapper    |
| **Sync**         | Windows Events (blocking)        | Spin lock (busy-wait)  |
| **Direction**    | Bidirectional (separate buffers) | Unidirectional         |
| **Dependencies** | `windows` only                   | 4 crates               |
| **Use case**     | Message passing                  | Data storage for share |

## License

MIT
