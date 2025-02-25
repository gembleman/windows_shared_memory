use windows_shared_memory::{Client, ReceiveMessage, Server};

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
