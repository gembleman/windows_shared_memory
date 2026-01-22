//! Server example for cross-architecture shared memory test
//! Build with: cargo build --example server [--target TARGET]
//!
//! Usage: server [SHM_NAME] [BUFFER_SIZE_KB]
//! Example: server Local\\MySharedMemory 64

use std::env;
use windows_shared_memory::{ReceiveMessage, Server, DEFAULT_BUFFER_SIZE};

fn main() {
    let arch = if cfg!(target_pointer_width = "64") {
        "64-bit"
    } else {
        "32-bit"
    };

    println!("[Server {}] Starting...", arch);

    // Get shared memory name from args or use default
    let shm_name = env::args().nth(1);
    let shm_name_ref = shm_name.as_deref();

    // Get buffer size from args or use default (in KB)
    let buffer_size = env::args()
        .nth(2)
        .and_then(|s| s.parse::<usize>().ok())
        .map(|kb| kb * 1024)
        .unwrap_or(DEFAULT_BUFFER_SIZE);

    let server = match Server::with_buffer_size(shm_name_ref, buffer_size) {
        Ok(s) => {
            println!(
                "[Server {}] Created shared memory successfully (buffer: {} KB)",
                arch,
                s.buffer_size() / 1024
            );
            s
        }
        Err(e) => {
            eprintln!("[Server {}] Failed to create server: {:?}", arch, e);
            std::process::exit(1);
        }
    };

    // Send a message to client
    let msg = format!("Hello from {} server!", arch);
    match server.send(msg.as_bytes()) {
        Ok(_) => println!("[Server {}] Sent: {}", arch, msg),
        Err(e) => {
            eprintln!("[Server {}] Failed to send: {:?}", arch, e);
            std::process::exit(1);
        }
    }

    // Wait for response from client (30 second timeout)
    println!("[Server {}] Waiting for client message...", arch);
    match server.receive(Some(30000)) {
        ReceiveMessage::Message(received) => {
            println!("[Server {}] Received: {}", arch, received);
            println!(
                "[Server {}] SUCCESS - Cross-architecture communication works!",
                arch
            );
        }
        ReceiveMessage::Timeout => {
            eprintln!("[Server {}] Timeout waiting for client", arch);
            std::process::exit(1);
        }
        ReceiveMessage::Exit => {
            println!("[Server {}] Client sent exit signal", arch);
        }
        ReceiveMessage::MessageError(e) => {
            eprintln!("[Server {}] Message error occurred: {}", arch, e);
            std::process::exit(1);
        }
    }

    // Send close signal
    let _ = server.send_close();
    println!("[Server {}] Done.", arch);
}
