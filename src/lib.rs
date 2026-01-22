//! # Windows Shared Memory
//!
//! A Rust library for working with shared memory on Windows systems.
//! This library provides a simple API for inter-process communication (IPC)
//! using Windows shared memory and events.
//!
//! ## Features
//!
//! - Server-client architecture for shared memory communication
//! - Support for 32-bit and 64-bit processes communication
//! - Synchronization using Windows events
//! - Thread-safe operations using atomic operations
//!
//! ## Example
//!
//! ```no_run
//! use windows_shared_memory::{Server, Client, ReceiveMessage};
//!
//! // Server side
//! let server = Server::new(None).unwrap();
//! server.send(b"Hello from server").unwrap();
//!
//! // Client side
//! let client = Client::new(None).unwrap();
//! client.send(b"Hello from client").unwrap();
//!
//! if let ReceiveMessage::Message(msg) = server.receive(Some(1000)) {
//!     println!("Server received: {}", msg);
//! }
//! ```

mod client;
mod server;
mod shared_memory;
mod skima;
mod utils;

pub use client::*;
pub use server::*;
pub use shared_memory::*;
pub use skima::*;
pub use utils::*;
