use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::*;

/// 문자열을 PCWSTR로 변환
pub fn str_to_pcwstr(s: &str) -> PCWSTR {
    let encoded: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    PCWSTR::from_raw(encoded.as_ptr())
}

/// 이벤트 객체 열기
pub fn open_event(event_name: &str) -> Result<HANDLE> {
    let event_name_pcwstr = str_to_pcwstr(event_name);
    let h_event = unsafe { OpenEventW(EVENT_ALL_ACCESS, false, event_name_pcwstr) }?;

    if h_event.is_invalid() {
        return Err(windows::core::Error::from_win32());
    }

    Ok(h_event)
}

/// 이벤트 객체 생성
pub fn create_event(event_name: &str) -> Result<HANDLE> {
    let event_name_pcwstr = str_to_pcwstr(event_name);
    let h_event = unsafe { CreateEventW(None, false, false, event_name_pcwstr) }?;

    if h_event.is_invalid() {
        return Err(windows::core::Error::from_win32());
    }

    Ok(h_event)
}

/// 이벤트 대기
pub fn wait_for_event(handle: HANDLE, timeout_ms: Option<u32>) -> Result<bool> {
    let timeout = timeout_ms.unwrap_or(INFINITE);
    let result = unsafe { WaitForSingleObject(handle, timeout) };

    match result {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        _ => Err(windows::core::Error::from_win32()),
    }
}
