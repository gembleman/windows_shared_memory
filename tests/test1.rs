use windows_shared_memory::{Client, ReceiveBytes, ReceiveMessage, Server, DEFAULT_BUFFER_SIZE};

#[test]
fn simple_test() {
    let server = Server::new(None).unwrap();
    let data = "hello i'am server".as_bytes();
    let _ = server.send(&data);

    let client = Client::new(None).unwrap();
    let data = "hello i'am client".as_bytes();
    let _ = client.send(&data);

    if let ReceiveMessage::Message(recv_mess) = client.receive(Some(30)) {
        println!("클라이언트가 받은 메세지: {:?}", recv_mess);
    }

    if let ReceiveMessage::Message(recv_mess) = server.receive(Some(30)) {
        println!("서버가 받은 메세지: {:?}", recv_mess);
    }
}

#[test]
fn test_default_buffer_size() {
    let server = Server::new(Some("Local\\TestDefaultBuffer")).unwrap();
    assert_eq!(server.buffer_size(), DEFAULT_BUFFER_SIZE);
    assert_eq!(server.buffer_size(), 16 * 1024);

    let client = Client::new(Some("Local\\TestDefaultBuffer")).unwrap();
    assert_eq!(client.buffer_size(), DEFAULT_BUFFER_SIZE);
}

#[test]
fn test_custom_buffer_size_small() {
    let custom_size = 4 * 1024; // 4KB
    let server = Server::with_buffer_size(Some("Local\\TestSmallBuffer"), custom_size).unwrap();
    assert_eq!(server.buffer_size(), custom_size);

    let client = Client::new(Some("Local\\TestSmallBuffer")).unwrap();
    assert_eq!(client.buffer_size(), custom_size);

    // Test send/receive with small buffer
    let msg = "Small buffer test message";
    server.send(msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
        assert_eq!(received, msg);
    } else {
        panic!("Failed to receive message with small buffer");
    }
}

#[test]
fn test_custom_buffer_size_large() {
    let custom_size = 64 * 1024; // 64KB
    let server = Server::with_buffer_size(Some("Local\\TestLargeBuffer"), custom_size).unwrap();
    assert_eq!(server.buffer_size(), custom_size);

    let client = Client::new(Some("Local\\TestLargeBuffer")).unwrap();
    assert_eq!(client.buffer_size(), custom_size);

    // Test send/receive with large buffer
    let msg = "Large buffer test message";
    server.send(msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
        assert_eq!(received, msg);
    } else {
        panic!("Failed to receive message with large buffer");
    }
}

#[test]
fn test_large_data_transfer() {
    let custom_size = 32 * 1024; // 32KB buffer
    let server = Server::with_buffer_size(Some("Local\\TestLargeData"), custom_size).unwrap();
    let client = Client::new(Some("Local\\TestLargeData")).unwrap();

    // Create a large message (20KB)
    let large_msg: String = "A".repeat(20 * 1024);
    server.send(large_msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
        assert_eq!(received.len(), 20 * 1024);
        assert_eq!(received, large_msg);
    } else {
        panic!("Failed to receive large message");
    }
}

#[test]
fn test_bidirectional_communication() {
    let server = Server::with_buffer_size(Some("Local\\TestBidirectional"), 8 * 1024).unwrap();
    let client = Client::new(Some("Local\\TestBidirectional")).unwrap();

    // Server to Client
    let server_msg = "Hello from server";
    server.send(server_msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
        assert_eq!(received, server_msg);
    } else {
        panic!("Client failed to receive from server");
    }

    // Client to Server
    let client_msg = "Hello from client";
    client.send(client_msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = server.receive(Some(100)) {
        assert_eq!(received, client_msg);
    } else {
        panic!("Server failed to receive from client");
    }
}

#[test]
fn test_receive_bytes() {
    let server = Server::with_buffer_size(Some("Local\\TestReceiveBytes"), 8 * 1024).unwrap();
    let client = Client::new(Some("Local\\TestReceiveBytes")).unwrap();

    // Send binary data
    let binary_data: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
    server.send(&binary_data).unwrap();

    if let ReceiveBytes::Bytes(received) = client.receive_bytes(Some(100)) {
        assert_eq!(received, binary_data);
    } else {
        panic!("Failed to receive binary data");
    }

    // Client sends binary data back
    let client_binary: Vec<u8> = vec![0xAA, 0xBB, 0xCC, 0xDD];
    client.send(&client_binary).unwrap();

    if let ReceiveBytes::Bytes(received) = server.receive_bytes(Some(100)) {
        assert_eq!(received, client_binary);
    } else {
        panic!("Server failed to receive binary data");
    }
}

#[test]
fn test_buffer_size_boundary() {
    let custom_size = 1024; // 1KB buffer
    let server = Server::with_buffer_size(Some("Local\\TestBoundary"), custom_size).unwrap();
    let client = Client::new(Some("Local\\TestBoundary")).unwrap();

    // Test exact buffer size
    let exact_msg: String = "X".repeat(1024);
    server.send(exact_msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
        assert_eq!(received.len(), 1024);
    } else {
        panic!("Failed to receive exact buffer size message");
    }

    // Test oversized data (should be truncated)
    let oversized_msg: String = "Y".repeat(2048);
    client.send(oversized_msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = server.receive(Some(100)) {
        // Should be truncated to buffer size
        assert_eq!(received.len(), 1024);
        assert!(received.chars().all(|c| c == 'Y'));
    } else {
        panic!("Failed to receive truncated message");
    }
}

#[test]
fn test_multiple_messages() {
    let server = Server::with_buffer_size(Some("Local\\TestMultiple"), 4 * 1024).unwrap();
    let client = Client::new(Some("Local\\TestMultiple")).unwrap();

    for i in 0..5 {
        let msg = format!("Message {}", i);
        server.send(msg.as_bytes()).unwrap();

        if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
            assert_eq!(received, msg);
        } else {
            panic!("Failed to receive message {}", i);
        }

        let reply = format!("Reply {}", i);
        client.send(reply.as_bytes()).unwrap();

        if let ReceiveMessage::Message(received) = server.receive(Some(100)) {
            assert_eq!(received, reply);
        } else {
            panic!("Failed to receive reply {}", i);
        }
    }
}

#[test]
fn test_send_close_signal() {
    let server = Server::with_buffer_size(Some("Local\\TestClose"), 4 * 1024).unwrap();
    let client = Client::new(Some("Local\\TestClose")).unwrap();

    // Send close signal
    server.send_close().unwrap();

    // Client should receive Exit
    match client.receive(Some(100)) {
        ReceiveMessage::Exit => {
            // Expected
        }
        other => {
            panic!("Expected Exit, got {:?}", other);
        }
    }
}

#[test]
fn test_very_small_buffer() {
    let custom_size = 64; // 64 bytes - very small
    let server = Server::with_buffer_size(Some("Local\\TestVerySmall"), custom_size).unwrap();
    let client = Client::new(Some("Local\\TestVerySmall")).unwrap();

    assert_eq!(server.buffer_size(), 64);
    assert_eq!(client.buffer_size(), 64);

    let small_msg = "Hi!";
    server.send(small_msg.as_bytes()).unwrap();

    if let ReceiveMessage::Message(received) = client.receive(Some(100)) {
        assert_eq!(received, small_msg);
    } else {
        panic!("Failed with very small buffer");
    }
}
