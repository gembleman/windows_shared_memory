use windows::core::Result;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::Threading::CreateEventW;
use windows::Win32::System::Threading::{OpenEventW, EVENT_ALL_ACCESS};

/// 문자열을 PCWSTR로 변환하는 유틸리티 함수
pub fn str_to_pcwstr(s: &str) -> PCWSTR {
    let encoded: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    PCWSTR::from_raw(encoded.as_ptr())
}

/// 이벤트 객체를 여는 함수 - 32비트 환경에서는 작동하지 않는다.
pub fn open_event(event_name: &str) -> Result<HANDLE> {
    let event_name_pcwstr = str_to_pcwstr(event_name);

    let h_event = unsafe { OpenEventW(EVENT_ALL_ACCESS, BOOL(0), event_name_pcwstr) }?;

    if h_event.is_invalid() {
        return Err(windows::core::Error::from_win32());
    }

    Ok(h_event)
}

/// 이벤트 객체를 생성하는 함수
pub fn create_event(event_name: &str) -> Result<HANDLE> {
    let event_name_pcwstr = str_to_pcwstr(event_name);

    let h_event = unsafe { CreateEventW(None, BOOL(0), BOOL(0), event_name_pcwstr) }?;

    if h_event.is_invalid() {
        return Err(windows::core::Error::from_win32());
    }

    Ok(h_event)
}
