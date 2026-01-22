//! Client example for cross-architecture shared memory test
//! Build with: cargo build --example client [--target TARGET]

use std::env;
use windows_shared_memory::{Client, ReceiveMessage};

fn main() {
    let arch = if cfg!(target_pointer_width = "64") {
        "64-bit"
    } else {
        "32-bit"
    };

    println!("[Client {}] Starting...", arch);

    // Get shared memory name from args or use default
    let shm_name = env::args().nth(1);
    let shm_name_ref = shm_name.as_deref();

    // Small delay to ensure server is ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    let client = match Client::new(shm_name_ref) {
        Ok(c) => {
            println!("[Client {}] Connected to shared memory successfully", arch);
            c
        }
        Err(e) => {
            eprintln!("[Client {}] Failed to connect: {:?}", arch, e);
            std::process::exit(1);
        }
    };

    // Receive message from server first (30 second timeout)
    println!("[Client {}] Waiting for server message...", arch);
    match client.receive(Some(30000)) {
        ReceiveMessage::Message(received) => {
            println!("[Client {}] Received: {}", arch, received);
        }
        ReceiveMessage::Timeout => {
            eprintln!("[Client {}] Timeout waiting for server", arch);
            std::process::exit(1);
        }
        ReceiveMessage::Exit => {
            println!("[Client {}] Server sent exit signal", arch);
            std::process::exit(0);
        }
        ReceiveMessage::MessageError(e) => {
            eprintln!("[Client {}] Message error occurred: {}", arch, e);
            std::process::exit(1);
        }
    }

    // Send response to server
    let msg = format!("Hello from {} client!", arch);
    match client.send(msg.as_bytes()) {
        Ok(_) => {
            println!("[Client {}] Sent: {}", arch, msg);
            println!("[Client {}] SUCCESS - Cross-architecture communication works!", arch);
        }
        Err(e) => {
            eprintln!("[Client {}] Failed to send: {:?}", arch, e);
            std::process::exit(1);
        }
    }

    println!("[Client {}] Done.", arch);
}
