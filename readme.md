# Windows Shared Memory for Rust

[![Crates.io](https://img.shields.io/crates/v/windows_shared_memory.svg)](https://crates.io/crates/windows_shared_memory)
[![Documentation](https://docs.rs/windows_shared_memory/badge.svg)](https://docs.rs/windows_shared_memory)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for inter-process communication (IPC) using Windows shared memory and events. This library was designed with support for communication between 32-bit and 64-bit processes.

## Features

- **Simple API**: Easy-to-use server-client architecture
- **Cross-architecture**: Supports communication between 32-bit and 64-bit processes
- **Synchronization**: Uses Windows events for efficient synchronization
- **Thread-safe**: Atomic operations ensure safe concurrent access
- **Rust 2024**: Built with the latest Rust edition

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
windows_shared_memory = "0.1.3"
```

## Usage

### Basic Example

```rust
use windows_shared_memory::{Server, Client, ReceiveMessage};

// Server process
fn server() {
    let server = Server::new(None).unwrap();
    server.send(b"Hello from server").unwrap();

    if let ReceiveMessage::Message(msg) = server.receive(Some(1000)) {
        println!("Server received: {}", msg);
    }
}

// Client process
fn client() {
    let client = Client::new(None).unwrap();
    client.send(b"Hello from client").unwrap();

    if let ReceiveMessage::Message(msg) = client.receive(Some(1000)) {
        println!("Client received: {}", msg);
    }
}
```

### Custom Shared Memory Path

```rust
use windows_shared_memory::Server;

let server = Server::new(Some("Local\\MyCustomMemory")).unwrap();
```

### Handling Timeouts

```rust
use windows_shared_memory::{Client, ReceiveMessage};

let client = Client::new(None).unwrap();

match client.receive(Some(5000)) {
    ReceiveMessage::Message(msg) => println!("Received: {}", msg),
    ReceiveMessage::Timeout => println!("No message received within 5 seconds"),
    ReceiveMessage::Exit => println!("Server closed the connection"),
    ReceiveMessage::MessageError(err) => eprintln!("Error: {}", err),
}
```

### Graceful Shutdown

```rust
use windows_shared_memory::Server;

let server = Server::new(None).unwrap();
// ... communication ...
server.send_close().unwrap(); // Notify clients to exit
```

## API Overview

### Server

- `Server::new(path: Option<&str>)` - Create a new server instance
- `server.send(data: &[u8])` - Send data to clients (max 16KB)
- `server.receive(timeout_ms: Option<u32>)` - Receive data from clients
- `server.send_close()` - Send close signal to clients

### Client

- `Client::new(path: Option<&str>)` - Connect to an existing server
- `client.send(data: &[u8])` - Send data to server (max 16KB)
- `client.receive(timeout_ms: Option<u32>)` - Receive data from server

### ReceiveMessage

- `Message(String)` - Successfully received a message
- `Timeout` - Operation timed out
- `Exit` - Connection closed by peer
- `MessageError(String)` - An error occurred

## Technical Details

- **Buffer Size**: 16 KB per direction (configurable via `BUFFER_SIZE` constant)
- **Synchronization**: Uses Windows named events for signaling
- **Memory Layout**: `#[repr(C, align(8))]` for cross-architecture compatibility
- **Default Memory Path**: `Local\\MySharedMemory`
- **Event Names**: `Local\\MyEventS2C` and `Local\\MyEventC2S`

## Platform Support

- Windows only (requires `windows` crate 0.62+)
- Tested on Windows 10/11
- Supports both 32-bit and 64-bit architectures

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Repository

[https://github.com/gembleman/windows_shared_memory](https://github.com/gembleman/windows_shared_memory)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
