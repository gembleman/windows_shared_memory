use windows_shared_memory::{RecieveMessage, Server};

fn main() {
    let server = Server::new(None).unwrap();
    let data = "hello i'am server".as_bytes();
    let _ = server.send_s2c(&data);

    if let RecieveMessage::Message(recv_mess) = server.recv_c2s(true) {
        println!("{:?}", recv_mess);
    }
}
