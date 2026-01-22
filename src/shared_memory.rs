use crate::{ReceiveBytes, ReceiveMessage, SharedDataHeader};
use std::sync::atomic::Ordering;
use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{SetEvent, WaitForSingleObject};

/// Writes data to shared memory
///
/// # Safety
///
/// The caller must ensure:
/// - `header` is a valid, properly aligned pointer to initialized SharedDataHeader
/// - No other threads are concurrently writing to the same buffer
/// - `event_handle` is a valid Windows event handle
/// - `buffer_size` matches the actual buffer size allocated after the header
pub unsafe fn write_to_shared_memory(
    header: *mut SharedDataHeader,
    buffer_size: usize,
    data: &[u8],
    is_server: bool,
    event_handle: HANDLE,
) -> Result<()> {
    unsafe {
        let base_ptr = header as *mut u8;

        let (flag, data_buffer_offset, data_len) = if is_server {
            (
                &(*header).flag_server,
                SharedDataHeader::offset_s2c(),
                &mut (*header).data_len_server_to_client,
            )
        } else {
            (
                &(*header).flag_client,
                SharedDataHeader::offset_c2s(buffer_size),
                &mut (*header).data_len_client_to_server,
            )
        };

        let data_buffer = std::slice::from_raw_parts_mut(base_ptr.add(data_buffer_offset), buffer_size);

        // Initialize and copy data buffer
        data_buffer.fill(0);
        let copy_len = std::cmp::min(data.len(), buffer_size);
        data_buffer[..copy_len].copy_from_slice(&data[..copy_len]);
        *data_len = copy_len as u32;

        // Set flag (1: data sent)
        flag.store(1, Ordering::Release);

        // Set event signal
        SetEvent(event_handle)?;

        Ok(())
    }
}

/// Reads data from shared memory as a String
///
/// # Safety
///
/// The caller must ensure:
/// - `header` is a valid, properly aligned pointer to initialized SharedDataHeader
/// - No other threads are concurrently reading from the same buffer
/// - `event_handle` is a valid Windows event handle
/// - `buffer_size` matches the actual buffer size allocated after the header
pub unsafe fn read_from_shared_memory(
    header: *mut SharedDataHeader,
    buffer_size: usize,
    is_server_reading: bool,
    timeout_ms: Option<u32>,
    event_handle: HANDLE,
) -> ReceiveMessage {
    unsafe {
        // Wait for event
        if let Some(timeout) = timeout_ms {
            match WaitForSingleObject(event_handle, timeout) {
                WAIT_OBJECT_0 => {}
                WAIT_TIMEOUT => return ReceiveMessage::Timeout,
                _ => return ReceiveMessage::MessageError("Event wait failed".to_string()),
            }
        }

        let base_ptr = header as *const u8;

        let (flag, data_buffer_offset, data_len) = if is_server_reading {
            // Server reading: data from client
            (
                &(*header).flag_client,
                SharedDataHeader::offset_c2s(buffer_size),
                (*header).data_len_client_to_server as usize,
            )
        } else {
            // Client reading: data from server
            (
                &(*header).flag_server,
                SharedDataHeader::offset_s2c(),
                (*header).data_len_server_to_client as usize,
            )
        };

        let data_buffer = std::slice::from_raw_parts(base_ptr.add(data_buffer_offset), buffer_size);

        // State - 0: waiting, 1: data sent, 2: data received, 3: exit
        match flag.load(Ordering::Acquire) {
            1 => {
                // Read only the valid data length
                let valid_len = std::cmp::min(data_len, buffer_size);
                let valid_data = &data_buffer[..valid_len];
                let message = match String::from_utf8(valid_data.to_vec()) {
                    Ok(s) => s,
                    Err(_) => return ReceiveMessage::MessageError("UTF-8 conversion failed".to_string()),
                };

                // Mark data as received (2)
                flag.store(2, Ordering::Release);
                ReceiveMessage::Message(message)
            }
            3 => ReceiveMessage::Exit,
            2 => ReceiveMessage::Timeout,
            0 => ReceiveMessage::Timeout,
            _ => ReceiveMessage::MessageError("Unknown state".to_string()),
        }
    }
}

/// Reads raw bytes from shared memory
///
/// # Safety
///
/// The caller must ensure:
/// - `header` is a valid, properly aligned pointer to initialized SharedDataHeader
/// - No other threads are concurrently reading from the same buffer
/// - `event_handle` is a valid Windows event handle
/// - `buffer_size` matches the actual buffer size allocated after the header
pub unsafe fn read_bytes_from_shared_memory(
    header: *mut SharedDataHeader,
    buffer_size: usize,
    is_server_reading: bool,
    timeout_ms: Option<u32>,
    event_handle: HANDLE,
) -> ReceiveBytes {
    unsafe {
        // Wait for event
        if let Some(timeout) = timeout_ms {
            match WaitForSingleObject(event_handle, timeout) {
                WAIT_OBJECT_0 => {}
                WAIT_TIMEOUT => return ReceiveBytes::Timeout,
                _ => return ReceiveBytes::Error("Event wait failed".to_string()),
            }
        }

        let base_ptr = header as *const u8;

        let (flag, data_buffer_offset, data_len) = if is_server_reading {
            (
                &(*header).flag_client,
                SharedDataHeader::offset_c2s(buffer_size),
                (*header).data_len_client_to_server as usize,
            )
        } else {
            (
                &(*header).flag_server,
                SharedDataHeader::offset_s2c(),
                (*header).data_len_server_to_client as usize,
            )
        };

        let data_buffer = std::slice::from_raw_parts(base_ptr.add(data_buffer_offset), buffer_size);

        match flag.load(Ordering::Acquire) {
            1 => {
                let valid_len = std::cmp::min(data_len, buffer_size);
                let bytes = data_buffer[..valid_len].to_vec();
                flag.store(2, Ordering::Release);
                ReceiveBytes::Bytes(bytes)
            }
            3 => ReceiveBytes::Exit,
            2 => ReceiveBytes::Timeout,
            0 => ReceiveBytes::Timeout,
            _ => ReceiveBytes::Error("Unknown state".to_string()),
        }
    }
}
