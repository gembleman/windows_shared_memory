use std::sync::atomic::AtomicU32;

/// Maximum buffer size for data transfer (16 KB)
pub const BUFFER_SIZE: usize = 16 * 1024;

/// Shared memory data structure used for IPC.
///
/// This structure is designed to work across 32-bit and 64-bit processes
/// with proper alignment and explicit field ordering.
#[repr(C, align(8))]
pub struct SharedData {
    /// Server state flag - 0: waiting, 1: data sent, 2: data received, 3: exit
    pub flag_server: AtomicU32,
    /// Client state flag - 0: waiting, 1: data sent, 2: data received, 3: exit
    pub flag_client: AtomicU32,
    /// Length of data from server to client
    pub data_len_server_to_client: u32,
    /// Length of data from client to server
    pub data_len_client_to_server: u32,
    /// Data buffer from server to client
    pub data_server_to_client: [u8; BUFFER_SIZE],
    /// Data buffer from client to server
    pub data_client_to_server: [u8; BUFFER_SIZE],
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

/// Result of receiving a message from shared memory.
#[derive(Debug)]
pub enum ReceiveMessage {
    /// The sender requested to exit/close the connection
    Exit,
    /// Successfully received a message
    Message(String),
    /// An error occurred while receiving the message
    MessageError(String),
    /// The receive operation timed out
    Timeout,
}
