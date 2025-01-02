use windows_shared_memory::{Client, RecieveMessage};

fn main() {
    let client = Client::new(None).unwrap();
    let data = "hello i'am client".as_bytes();
    let _ = client.send_c2s(&data);

    if let RecieveMessage::Message(recv_mess) = client.recv_s2c(true) {
        println!("{:?}", recv_mess);
    }
}
