use crate::{ReceiveMessage, SharedData, BUFFER_SIZE};
use std::sync::atomic::Ordering;
use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{SetEvent, WaitForSingleObject};

/// 공유 메모리에 데이터 쓰기
pub fn write_to_shared_memory(
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

        // 데이터 버퍼 초기화 및 복사
        data_buffer.fill(0);
        let copy_len = std::cmp::min(data.len(), BUFFER_SIZE);
        data_buffer[..copy_len].copy_from_slice(&data[..copy_len]);
        *data_len = copy_len as u32;

        // 플래그 설정 (1: 데이터 전송)
        flag.store(1, Ordering::Release);

        // 이벤트 신호 설정
        SetEvent(event_handle)?;
    }
    Ok(())
}

/// 공유 메모리에서 데이터 읽기
pub fn read_from_shared_memory(
    shared_data: *mut SharedData,
    is_server_reading: bool,
    timeout_ms: Option<u32>,
    event_handle: HANDLE,
) -> ReceiveMessage {
    // 이벤트 대기
    if let Some(timeout) = timeout_ms {
        match unsafe { WaitForSingleObject(event_handle, timeout) } {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => return ReceiveMessage::Timeout,
            _ => return ReceiveMessage::MessageError("이벤트 대기 실패".to_string()),
        }
    }

    unsafe {
        let (flag, data_buffer, data_len) = if is_server_reading {
            // 서버가 읽는 경우 클라이언트에서 받은 데이터
            (
                &(*shared_data).flag_client,
                &(*shared_data).data_client_to_server,
                (*shared_data).data_len_client_to_server as usize,
            )
        } else {
            // 클라이언트가 읽는 경우 서버에서 받은 데이터
            (
                &(*shared_data).flag_server,
                &(*shared_data).data_server_to_client,
                (*shared_data).data_len_server_to_client as usize,
            )
        };

        // 상태 - 0: 대기, 1: 데이터 전송, 2: 데이터 수신 완료, 3: 종료
        match flag.load(Ordering::Acquire) {
            1 => {
                // 실제 데이터 길이만큼만 읽기
                let valid_data = &data_buffer[..data_len];
                let message = match String::from_utf8(valid_data.to_vec()) {
                    Ok(s) => s,
                    Err(_) => return ReceiveMessage::MessageError("UTF-8 변환 실패".to_string()),
                };

                // 데이터 수신 완료 표시 (2)
                flag.store(2, Ordering::Release);
                ReceiveMessage::Message(message)
            }
            3 => ReceiveMessage::Exit,
            2 => ReceiveMessage::Timeout,
            0 => ReceiveMessage::Timeout,
            _ => ReceiveMessage::MessageError("알 수 없는 상태".to_string()),
        }
    }
}
