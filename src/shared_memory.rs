use crate::{ReceiveMessage, SharedData, BUFFER_SIZE};
use std::sync::atomic::Ordering;
use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{SetEvent, WaitForSingleObject};

/// Writes data to shared memory
///
/// # Safety
///
/// The caller must ensure:
/// - `shared_data` is a valid, properly aligned pointer to initialized SharedData
/// - No other threads are concurrently writing to the same buffer
/// - `event_handle` is a valid Windows event handle
pub unsafe fn write_to_shared_memory(
    shared_data: *mut SharedData,
    data: &[u8],
    is_server: bool,
    event_handle: HANDLE,
) -> Result<()> {
    unsafe {
        let (flag, data_buffer, data_len) = if is_server {
            (
                &(*shared_data).flag_server,
                &mut (*shared_data).data_server_to_client,
                &mut (*shared_data).data_len_server_to_client,
            )
        } else {
            (
                &(*shared_data).flag_client,
                &mut (*shared_data).data_client_to_server,
                &mut (*shared_data).data_len_client_to_server,
            )
        };

        // Initialize and copy data buffer
        data_buffer.fill(0);
        let copy_len = std::cmp::min(data.len(), BUFFER_SIZE);
        data_buffer[..copy_len].copy_from_slice(&data[..copy_len]);
        *data_len = copy_len as u32;

        // Set flag (1: data sent)
        flag.store(1, Ordering::Release);

        // Set event signal
        SetEvent(event_handle)?;

        Ok(())
    }
}

/// Reads data from shared memory
///
/// # Safety
///
/// The caller must ensure:
/// - `shared_data` is a valid, properly aligned pointer to initialized SharedData
/// - No other threads are concurrently reading from the same buffer
/// - `event_handle` is a valid Windows event handle
pub unsafe fn read_from_shared_memory(
    shared_data: *mut SharedData,
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

        let (flag, data_buffer, data_len) = if is_server_reading {
            // Server reading: data from client
            (
                &(*shared_data).flag_client,
                &(*shared_data).data_client_to_server,
                (*shared_data).data_len_client_to_server as usize,
            )
        } else {
            // Client reading: data from server
            (
                &(*shared_data).flag_server,
                &(*shared_data).data_server_to_client,
                (*shared_data).data_len_server_to_client as usize,
            )
        };

        // State - 0: waiting, 1: data sent, 2: data received, 3: exit
        match flag.load(Ordering::Acquire) {
            1 => {
                // Read only the valid data length
                let valid_data = &data_buffer[..data_len];
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
