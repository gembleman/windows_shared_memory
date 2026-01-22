use crate::{
    open_event, read_from_shared_memory, str_to_pcwstr, write_to_shared_memory, ReceiveBytes,
    ReceiveMessage, SharedDataHeader,
};
use windows::core::Result;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::*,
};

/// Client instance for shared memory communication.
///
/// The client connects to an existing shared memory created by the server.
pub struct Client {
    header_address: *mut SharedDataHeader,
    buffer_size: usize,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Client {
    /// Creates a new client instance and connects to shared memory.
    ///
    /// The buffer size is automatically read from the shared memory header
    /// that was set by the server.
    ///
    /// # Arguments
    ///
    /// * `mapping_path` - Optional custom path for the shared memory mapping.
    ///   If None, uses "Local\\MySharedMemory" as default.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windows_shared_memory::Client;
    ///
    /// let client = Client::new(None).unwrap();
    /// ```
    pub fn new(mapping_path: Option<&str>) -> Result<Self> {
        let mapping_name = mapping_path.unwrap_or("Local\\MySharedMemory");
        let mapping_name_pcwstr = str_to_pcwstr(mapping_name);

        // Open file mapping object
        let h_map_file =
            unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, false, &mapping_name_pcwstr)? };

        // First, map only the header to read buffer size
        let header_size = std::mem::size_of::<SharedDataHeader>();
        let p_buf_header = unsafe {
            MapViewOfFile(
                h_map_file,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                header_size,
            )
        };

        if p_buf_header.Value.is_null() {
            unsafe { CloseHandle(h_map_file)? };
            return Err(windows::core::Error::from_thread());
        }

        // Read buffer size from header
        let buffer_size = unsafe { (*(p_buf_header.Value as *const SharedDataHeader)).buffer_size as usize };

        // Unmap header-only view
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: p_buf_header.Value,
            })?;
        }

        // Now map the full shared memory
        let total_size = SharedDataHeader::total_size(buffer_size);
        let p_buf = unsafe {
            MapViewOfFile(
                h_map_file,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                total_size,
            )
        };

        if p_buf.Value.is_null() {
            unsafe { CloseHandle(h_map_file)? };
            return Err(windows::core::Error::from_thread());
        }

        // Open event objects
        let h_event_s2c = open_event("Local\\MyEventS2C")?;
        let h_event_c2s = open_event("Local\\MyEventC2S")?;

        Ok(Self {
            header_address: p_buf.Value as *mut SharedDataHeader,
            buffer_size,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    /// Returns the buffer size for this client (set by the server).
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Sends data to the server.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice to send. Maximum size is the configured buffer size.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use windows_shared_memory::Client;
    /// # let client = Client::new(None).unwrap();
    /// client.send(b"Hello, server!").unwrap();
    /// ```
    pub fn send(&self, data: &[u8]) -> Result<()> {
        unsafe {
            write_to_shared_memory(
                self.header_address,
                self.buffer_size,
                data,
                false,
                self.h_event_c2s,
            )
        }
    }

    /// Receives data from the server as a String.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Optional timeout in milliseconds. If None, waits indefinitely.
    ///
    /// # Returns
    ///
    /// Returns a ReceiveMessage enum containing the message or status.
    pub fn receive(&self, timeout_ms: Option<u32>) -> ReceiveMessage {
        unsafe {
            read_from_shared_memory(
                self.header_address,
                self.buffer_size,
                false,
                timeout_ms,
                self.h_event_s2c,
            )
        }
    }

    /// Receives raw bytes from the server.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Optional timeout in milliseconds. If None, waits indefinitely.
    ///
    /// # Returns
    ///
    /// Returns a ReceiveBytes enum containing the bytes or status.
    pub fn receive_bytes(&self, timeout_ms: Option<u32>) -> ReceiveBytes {
        unsafe {
            crate::read_bytes_from_shared_memory(
                self.header_address,
                self.buffer_size,
                false,
                timeout_ms,
                self.h_event_s2c,
            )
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.header_address as *mut _,
            }) {
                eprintln!("Failed to unmap shared memory: {:?}", e);
            }

            if let Err(e) = CloseHandle(self.h_event_s2c) {
                eprintln!("Failed to close event handle (s2c): {:?}", e);
            }

            if let Err(e) = CloseHandle(self.h_event_c2s) {
                eprintln!("Failed to close event handle (c2s): {:?}", e);
            }

            if let Err(e) = CloseHandle(self.h_map_file) {
                eprintln!("Failed to close file mapping handle: {:?}", e);
            }
        }
    }
}
