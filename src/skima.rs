use std::sync::atomic::AtomicU32;

/// Default buffer size for data transfer (16 KB)
pub const DEFAULT_BUFFER_SIZE: usize = 16 * 1024;

/// Header structure for shared memory.
///
/// This structure contains metadata and is placed at the beginning of shared memory.
/// It is designed to work across 32-bit and 64-bit processes.
#[repr(C, align(8))]
pub struct SharedDataHeader {
    /// Buffer size for each direction
    pub buffer_size: u32,
    /// Server state flag - 0: waiting, 1: data sent, 2: data received, 3: exit
    pub flag_server: AtomicU32,
    /// Client state flag - 0: waiting, 1: data sent, 2: data received, 3: exit
    pub flag_client: AtomicU32,
    /// Length of data from server to client
    pub data_len_server_to_client: u32,
    /// Length of data from client to server
    pub data_len_client_to_server: u32,
    /// Padding for alignment
    _padding: u32,
}

impl SharedDataHeader {
    /// Creates a new header with the specified buffer size.
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size: buffer_size as u32,
            flag_server: AtomicU32::new(0),
            flag_client: AtomicU32::new(0),
            data_len_server_to_client: 0,
            data_len_client_to_server: 0,
            _padding: 0,
        }
    }

    /// Returns the total size of shared memory needed for the given buffer size.
    pub fn total_size(buffer_size: usize) -> usize {
        std::mem::size_of::<SharedDataHeader>() + buffer_size * 2
    }

    /// Returns the offset to the server-to-client data buffer.
    pub fn offset_s2c() -> usize {
        std::mem::size_of::<SharedDataHeader>()
    }

    /// Returns the offset to the client-to-server data buffer.
    pub fn offset_c2s(buffer_size: usize) -> usize {
        std::mem::size_of::<SharedDataHeader>() + buffer_size
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

/// Result of receiving raw bytes from shared memory.
#[derive(Debug)]
pub enum ReceiveBytes {
    /// The sender requested to exit/close the connection
    Exit,
    /// Successfully received bytes
    Bytes(Vec<u8>),
    /// An error occurred while receiving
    Error(String),
    /// The receive operation timed out
    Timeout,
}
