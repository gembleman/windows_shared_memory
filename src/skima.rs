const BUFFER_SIZE: usize = 16 * 1024;

#[repr(C, align(8))]
pub struct SharedData {
    pub flag_server: u32, // 서버의 상태 플래그 - 0: 대기, 1: 데이터 전송, 2: 데이터 수신 완료, 3: 종료
    pub flag_client: u32, // 클라이언트의 상태 플래그 - 0: 대기, 1: 데이터 전송, 2: 데이터 수신 완료, 3: 종료
    pub data_server_to_client: [u8; BUFFER_SIZE], // 서버 -> 클라이언트 데이터
    // 16 * 1024은 16KB를 의미합니다. 16KB를 공유 메모리의 최대 크기로 설정했습니다.
    pub data_client_to_server: [u8; BUFFER_SIZE], // 클라이언트 -> 서버 데이터
}
impl Default for SharedData {
    fn default() -> Self {
        let data = Self {
            flag_server: 0,
            flag_client: 0,
            data_server_to_client: [0; BUFFER_SIZE],
            data_client_to_server: [0; BUFFER_SIZE],
        };

        data
    }
}
impl SharedData {
    pub fn new() -> Self {
        Self::default()
    }
}

pub enum RecieveMessage {
    Exit,
    Message(String),
    Error,
}
