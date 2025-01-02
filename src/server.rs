use crate::{create_event, str_to_pcwstr, RecieveMessage, SharedData};
use std::mem::size_of;

use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{SetEvent, WaitForSingleObject, INFINITE};
pub struct Server {
    shared_data_address: *mut SharedData,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Server {
    pub fn new(mapping_path: Option<&str>) -> Result<Self> {
        // 공유 메모리 이름 설정
        let mapping_name = str_to_pcwstr(mapping_path.unwrap_or("Local\\MySharedMemory"));

        // 파일 매핑 객체 생성
        let h_map_file = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                size_of::<SharedData>() as u32,
                mapping_name,
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
            return Err(windows::core::Error::from_win32());
        }

        // 이벤트 객체 생성 (서버 -> 클라이언트)
        let h_event_s2c = create_event("Local\\MyEventS2C")?;
        // 이벤트 객체 생성 (클라이언트 -> 서버)
        let h_event_c2s = create_event("Local\\MyEventC2S")?;

        Ok(Self {
            shared_data_address: p_buf.Value as *mut SharedData,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    pub fn send_close_s2c(&self) -> Result<()> {
        unsafe {
            // 클라이언트 종료 플래그 설정
            (*self.shared_data_address).flag_server = 3;

            // 클라이언트 종료 이벤트 신호 설정
            SetEvent(self.h_event_s2c)?;
        }

        Ok(())
    }

    pub fn send_s2c(&self, data: &[u8]) -> Result<()> {
        // 서버 -> 클라이언트로 데이터 전송 준비
        unsafe {
            // 데이터 버퍼 초기화
            (*self.shared_data_address).data_server_to_client.fill(0);

            // 메시지를 데이터 버퍼에 복사
            (*self.shared_data_address).data_server_to_client[..data.len()].copy_from_slice(data);

            // 서버 -> 클라이언트 플래그 설정
            (*self.shared_data_address).flag_server = 1;

            // 서버 -> 클라이언트 이벤트 신호 설정
            SetEvent(self.h_event_s2c)?;
        }

        Ok(())
    }

    /// 클라이언트 -> 서버 데이터 수신 단, WaitForSingleObject로 이벤트 객체를 기다리는 방식이므로 이 함수를 호출하면 클라이언트가 데이터를 보낼 때까지 블로킹됩니다.
    /// 블로킹되지 않으려면, wait 인자를 false로 설정하고, 이벤트 객체를 직접 사용하십시오.
    pub fn recv_c2s(&self, wait: bool) -> RecieveMessage {
        // 클라이언트 -> 서버 데이터 준비를 기다림
        if wait {
            unsafe {
                WaitForSingleObject(self.h_event_c2s, INFINITE);
            }
        }

        // 클라이언트 -> 서버 데이터 읽기
        unsafe {
            if (*self.shared_data_address).flag_client == 1 {
                let data = &(*self.shared_data_address).data_client_to_server;
                let message = String::from_utf8_lossy(&data[..])
                    .trim_end_matches('\0')
                    .to_string();

                // 데이터 수신 완료 표시
                (*self.shared_data_address).flag_client = 2;

                RecieveMessage::Message(message)
            } else if (*self.shared_data_address).flag_client == 3 {
                RecieveMessage::Exit
            } else {
                RecieveMessage::Error
            }
        }
    }
}
impl Drop for Server {
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
