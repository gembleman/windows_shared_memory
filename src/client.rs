use crate::{
    open_event, read_from_shared_memory, str_to_pcwstr, write_to_shared_memory, ReceiveMessage,
    SharedData,
};
use std::mem::size_of;
use windows::core::Result;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::*,
};

/// Client instance for shared memory communication.
///
/// The client connects to an existing shared memory created by the server.
pub struct Client {
    shared_data_address: *mut SharedData,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Client {
    /// Creates a new client instance and connects to shared memory.
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
        // Set shared memory name
        let mapping_name = mapping_path.unwrap_or("Local\\MySharedMemory");
        let mapping_name_pcwstr = str_to_pcwstr(mapping_name);

        // Open file mapping object
        let h_map_file =
            unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, false, &mapping_name_pcwstr)? };

        // 공유 메모리 매핑
        let p_buf = unsafe {
            MapViewOfFile(
                h_map_file,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                size_of::<SharedData>(),
            )
        };

        if p_buf.Value.is_null() {
            unsafe { CloseHandle(h_map_file)? };
            return Err(windows::core::Error::from_thread());
        }

        // 이벤트 객체 열기
        let h_event_s2c = open_event("Local\\MyEventS2C")?;
        let h_event_c2s = open_event("Local\\MyEventC2S")?;

        Ok(Self {
            shared_data_address: p_buf.Value as *mut SharedData,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    /// Sends data to the server.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice to send. Maximum size is 16KB (BUFFER_SIZE).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use windows_shared_memory::Client;
    /// # let client = Client::new(None).unwrap();
    /// client.send(b"Hello, server!").unwrap();
    /// ```
    pub fn send(&self, data: &[u8]) -> Result<()> {
        unsafe { write_to_shared_memory(self.shared_data_address, data, false, self.h_event_c2s) }
    }

    /// Receives data from the server.
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
                self.shared_data_address,
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
                Value: self.shared_data_address as *mut _,
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
