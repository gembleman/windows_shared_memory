use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::*;
use windows::core::{Result, HSTRING};

/// Converts a string to PCWSTR
pub fn str_to_pcwstr(s: &str) -> HSTRING {
    HSTRING::from(s)
}

/// Opens an event object
pub fn open_event(event_name: &str) -> Result<HANDLE> {
    let event_name_pcwstr = str_to_pcwstr(event_name);
    let h_event = unsafe { OpenEventW(EVENT_ALL_ACCESS, false, &event_name_pcwstr) }?;

    if h_event.is_invalid() {
        return Err(windows::core::Error::from_thread());
    }

    Ok(h_event)
}

/// Creates an event object
pub fn create_event(event_name: &str) -> Result<HANDLE> {
    let event_name_pcwstr = str_to_pcwstr(event_name);
    let h_event = unsafe { CreateEventW(None, false, false, &event_name_pcwstr) }?;

    if h_event.is_invalid() {
        return Err(windows::core::Error::from_thread());
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
        _ => Err(windows::core::Error::from_thread()),
    }
}
