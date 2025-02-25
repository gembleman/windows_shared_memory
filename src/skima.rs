use std::sync::atomic::AtomicU32;

pub const BUFFER_SIZE: usize = 16 * 1024;

#[repr(C, align(8))]
pub struct SharedData {
    pub flag_server: AtomicU32, // 서버 상태 - 0: 대기, 1: 데이터 전송, 2: 데이터 수신 완료, 3: 종료
    pub flag_client: AtomicU32, // 클라이언트 상태 - 0: 대기, 1: 데이터 전송, 2: 데이터 수신 완료, 3: 종료
    pub data_len_server_to_client: usize, // 서버->클라이언트 데이터 길이
    pub data_len_client_to_server: usize, // 클라이언트->서버 데이터 길이
    pub data_server_to_client: [u8; BUFFER_SIZE], // 서버->클라이언트 데이터
    pub data_client_to_server: [u8; BUFFER_SIZE], // 클라이언트->서버 데이터
}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            flag_server: AtomicU32::new(0),
            flag_client: AtomicU32::new(0),
            data_len_server_to_client: 0,
            data_len_client_to_server: 0,
            data_server_to_client: [0; BUFFER_SIZE],
            data_client_to_server: [0; BUFFER_SIZE],
        }
    }
}

impl SharedData {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub enum ReceiveMessage {
    Exit,
    Message(String),
    MessageError(String),
    Timeout,
}
