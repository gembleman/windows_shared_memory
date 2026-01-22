use crate::{
    create_event, read_from_shared_memory, str_to_pcwstr, write_to_shared_memory, ReceiveMessage,
    SharedData,
};
use std::mem::size_of;
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
    shared_data_address: *mut SharedData,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Server {
    /// Creates a new server instance with shared memory.
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
        // Set shared memory name
        let mapping_name = mapping_path.unwrap_or("Local\\MySharedMemory");
        let mapping_name_pcwstr = str_to_pcwstr(mapping_name);

        // Create file mapping object
        let h_map_file = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                size_of::<SharedData>() as u32,
                &mapping_name_pcwstr,
            )?
        };

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

        // 새로운 SharedData 인스턴스 초기화
        unsafe {
            std::ptr::write(p_buf.Value as *mut SharedData, SharedData::new());
        }

        // 이벤트 객체 생성
        let h_event_s2c = create_event("Local\\MyEventS2C")?;
        let h_event_c2s = create_event("Local\\MyEventC2S")?;

        Ok(Self {
            shared_data_address: p_buf.Value as *mut SharedData,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    /// Sends a close signal to all connected clients.
    pub fn send_close(&self) -> Result<()> {
        unsafe {
            (*self.shared_data_address)
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
    /// * `data` - Byte slice to send. Maximum size is 16KB (BUFFER_SIZE).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use windows_shared_memory::Server;
    /// # let server = Server::new(None).unwrap();
    /// server.send(b"Hello, client!").unwrap();
    /// ```
    pub fn send(&self, data: &[u8]) -> Result<()> {
        unsafe { write_to_shared_memory(self.shared_data_address, data, true, self.h_event_s2c) }
    }

    /// Receives data from connected clients.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Optional timeout in milliseconds. If None, waits indefinitely.
    ///
    /// # Returns
    ///
    /// Returns a ReceiveMessage enum containing the message or status.
    pub fn receive(&self, timeout_ms: Option<u32>) -> ReceiveMessage {
        unsafe { read_from_shared_memory(self.shared_data_address, true, timeout_ms, self.h_event_c2s) }
    }
}

impl Drop for Server {
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
