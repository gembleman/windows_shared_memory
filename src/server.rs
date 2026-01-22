use crate::{
    create_event, read_from_shared_memory, str_to_pcwstr, write_to_shared_memory, ReceiveBytes,
    ReceiveMessage, SharedDataHeader, DEFAULT_BUFFER_SIZE,
};
use std::sync::atomic::Ordering;
use windows::core::Result;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::{Memory::*, Threading::*},
};

/// Server instance for shared memory communication.
///
/// The server creates the shared memory and events, and can communicate
/// with clients that connect to the same shared memory.
pub struct Server {
    header_address: *mut SharedDataHeader,
    buffer_size: usize,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Server {
    /// Creates a new server instance with shared memory using the default buffer size (16KB).
    ///
    /// # Arguments
    ///
    /// * `mapping_path` - Optional custom path for the shared memory mapping.
    ///   If None, uses "Local\\MySharedMemory" as default.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windows_shared_memory::Server;
    ///
    /// let server = Server::new(None).unwrap();
    /// ```
    pub fn new(mapping_path: Option<&str>) -> Result<Self> {
        Self::with_buffer_size(mapping_path, DEFAULT_BUFFER_SIZE)
    }

    /// Creates a new server instance with shared memory and a custom buffer size.
    ///
    /// # Arguments
    ///
    /// * `mapping_path` - Optional custom path for the shared memory mapping.
    ///   If None, uses "Local\\MySharedMemory" as default.
    /// * `buffer_size` - Size of each data buffer in bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use windows_shared_memory::Server;
    ///
    /// // Create server with 64KB buffer
    /// let server = Server::with_buffer_size(None, 64 * 1024).unwrap();
    /// ```
    pub fn with_buffer_size(mapping_path: Option<&str>, buffer_size: usize) -> Result<Self> {
        let mapping_name = mapping_path.unwrap_or("Local\\MySharedMemory");
        let mapping_name_pcwstr = str_to_pcwstr(mapping_name);

        let total_size = SharedDataHeader::total_size(buffer_size);

        // Create file mapping object
        let h_map_file = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                total_size as u32,
                &mapping_name_pcwstr,
            )?
        };

        // Map shared memory
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

        // Initialize header
        unsafe {
            std::ptr::write(
                p_buf.Value as *mut SharedDataHeader,
                SharedDataHeader::new(buffer_size),
            );
            // Zero out the data buffers
            let data_ptr = (p_buf.Value as *mut u8).add(SharedDataHeader::offset_s2c());
            std::ptr::write_bytes(data_ptr, 0, buffer_size * 2);
        }

        // Create event objects
        let h_event_s2c = create_event("Local\\MyEventS2C")?;
        let h_event_c2s = create_event("Local\\MyEventC2S")?;

        Ok(Self {
            header_address: p_buf.Value as *mut SharedDataHeader,
            buffer_size,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    /// Returns the buffer size for this server.
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Sends a close signal to all connected clients.
    pub fn send_close(&self) -> Result<()> {
        unsafe {
            (*self.header_address)
                .flag_server
                .store(3, Ordering::Release);
            SetEvent(self.h_event_s2c)?;
        }
        Ok(())
    }

    /// Sends data to connected clients.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice to send. Maximum size is the configured buffer size.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use windows_shared_memory::Server;
    /// # let server = Server::new(None).unwrap();
    /// server.send(b"Hello, client!").unwrap();
    /// ```
    pub fn send(&self, data: &[u8]) -> Result<()> {
        unsafe {
            write_to_shared_memory(
                self.header_address,
                self.buffer_size,
                data,
                true,
                self.h_event_s2c,
            )
        }
    }

    /// Receives data from connected clients as a String.
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
                true,
                timeout_ms,
                self.h_event_c2s,
            )
        }
    }

    /// Receives raw bytes from connected clients.
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
                true,
                timeout_ms,
                self.h_event_c2s,
            )
        }
    }
}

impl Drop for Server {
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
