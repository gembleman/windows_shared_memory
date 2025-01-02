use crate::{RecieveMessage, SharedData};
use std::mem::size_of;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE};
use windows::Win32::System::Memory::{
    MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    MEMORY_MAPPED_VIEW_ADDRESS,
};
use windows::Win32::System::Threading::{
    OpenEventW, SetEvent, WaitForSingleObject, EVENT_ALL_ACCESS, INFINITE,
};

/// 클라이언트 구조체 32비트용
pub struct Client {
    shared_data_address: *mut SharedData,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Client {
    pub fn new(mapping_path: Option<&str>) -> Result<Self> {
        // 공유 메모리 이름 설정
        //32비트 환경에서는 util::str_to_pcwstr 함수가 작동하지 않는다...
        let mapping_name: Vec<u16> = mapping_path
            .unwrap_or("Local\\MySharedMemory")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mapping_name_pcwstr = PCWSTR::from_raw(mapping_name.as_ptr());

        // println!("클라이언트: 공유 메모리에 접근합니다.");

        // 파일 매핑 객체 열기
        let h_map_file =
            unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, BOOL(0), mapping_name_pcwstr)? };

        // println!("클라이언트: 파일 매핑 객체를 열었습니다.");

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
            unsafe {
                CloseHandle(h_map_file)?;
            }
            return Err(windows::core::Error::from_win32());
        }

        // println!("클라이언트: 공유 메모리를 매핑했습니다.");

        // 이벤트 객체 열기 (서버 -> 클라이언트)
        let event_name_s2c: Vec<u16> = "Local\\MyEventS2C".encode_utf16().chain(Some(0)).collect();
        let event_name_s2c_pcwstr = PCWSTR::from_raw(event_name_s2c.as_ptr());

        let h_event_s2c = unsafe { OpenEventW(EVENT_ALL_ACCESS, BOOL(0), event_name_s2c_pcwstr)? };

        if h_event_s2c.is_invalid() {
            unsafe {
                UnmapViewOfFile(p_buf)?;
                CloseHandle(h_map_file)?;
            }
            return Err(windows::core::Error::from_win32());
        }

        // 이벤트 객체 열기 (클라이언트 -> 서버)
        let event_name_c2s: Vec<u16> = "Local\\MyEventC2S".encode_utf16().chain(Some(0)).collect();
        let event_name_c2s_pcwstr = PCWSTR::from_raw(event_name_c2s.as_ptr());

        let h_event_c2s = unsafe { OpenEventW(EVENT_ALL_ACCESS, BOOL(0), event_name_c2s_pcwstr)? };

        if h_event_c2s.is_invalid() {
            unsafe {
                UnmapViewOfFile(p_buf)?;
                CloseHandle(h_map_file)?;
                CloseHandle(h_event_s2c)?;
            }
            return Err(windows::core::Error::from_win32());
        }

        Ok(Self {
            shared_data_address: p_buf.Value as *mut SharedData,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    pub fn send_c2s(&self, data: &[u8]) -> Result<()> {
        // 클라이언트 -> 서버로 데이터 전송 준비
        unsafe {
            // 공유 메모리에 데이터 쓰기

            // 데이터 버퍼 초기화
            (*self.shared_data_address).data_client_to_server.fill(0);

            // 데이터 복사
            (*self.shared_data_address).data_client_to_server[..data.len()].copy_from_slice(data);

            // 클라이언트 -> 서버 플래그 설정 // 1: 데이터 준비 완료
            (*self.shared_data_address).flag_client = 1;

            // 클라이언트 -> 서버 이벤트 신호 설정
            SetEvent(self.h_event_c2s)?;
        }
        Ok(())
    }

    pub fn recv_s2c(&self, wait: bool) -> RecieveMessage {
        if wait {
            // 서버의 데이터 준비를 기다림
            unsafe {
                WaitForSingleObject(self.h_event_s2c, INFINITE);
            }
        }

        // 공유 메모리에 데이터 읽기
        unsafe {
            if (*self.shared_data_address).flag_server == 1 {
                let data = (*self.shared_data_address).data_server_to_client;
                let message = String::from_utf8_lossy(&data)
                    .trim_end_matches('\0')
                    .to_string();

                // 데이터 수신 완료 표시
                (*self.shared_data_address).flag_server = 2;

                RecieveMessage::Message(message)
            } else if (*self.shared_data_address).flag_server == 3 {
                RecieveMessage::Exit
            } else {
                RecieveMessage::Error
            }
        }
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.shared_data_address as *mut _,
            });

            let _ = CloseHandle(self.h_event_s2c);
            let _ = CloseHandle(self.h_event_c2s);
            let _ = CloseHandle(self.h_map_file);
        }
    }
}
