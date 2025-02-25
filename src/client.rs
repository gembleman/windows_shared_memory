use crate::{
    open_event, read_from_shared_memory, str_to_pcwstr, write_to_shared_memory, ReceiveMessage,
    SharedData,
};
use std::mem::size_of;
use windows::core::Result;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::*,
};

pub struct Client {
    shared_data_address: *mut SharedData,
    h_map_file: HANDLE,
    h_event_s2c: HANDLE,
    h_event_c2s: HANDLE,
}

impl Client {
    pub fn new(mapping_path: Option<&str>) -> Result<Self> {
        // 공유 메모리 이름 설정
        let mapping_name = mapping_path.unwrap_or("Local\\MySharedMemory");
        let mapping_name_pcwstr = str_to_pcwstr(mapping_name);

        // 파일 매핑 객체 열기
        let h_map_file =
            unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, false, mapping_name_pcwstr)? };

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

        // 이벤트 객체 열기
        let h_event_s2c = open_event("Local\\MyEventS2C")?;
        let h_event_c2s = open_event("Local\\MyEventC2S")?;

        Ok(Self {
            shared_data_address: p_buf.Value as *mut SharedData,
            h_map_file,
            h_event_s2c,
            h_event_c2s,
        })
    }

    pub fn send(&self, data: &[u8]) -> Result<()> {
        write_to_shared_memory(self.shared_data_address, data, false, self.h_event_c2s)
    }

    pub fn receive(&self, timeout_ms: Option<u32>) -> ReceiveMessage {
        read_from_shared_memory(
            self.shared_data_address,
            false,
            timeout_ms,
            self.h_event_s2c,
        )
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.shared_data_address as *mut _,
            }) {
                eprintln!("공유 메모리 매핑 해제 실패: {:?}", e);
            }

            if let Err(e) = CloseHandle(self.h_event_s2c) {
                eprintln!("이벤트 핸들 닫기 실패(s2c): {:?}", e);
            }

            if let Err(e) = CloseHandle(self.h_event_c2s) {
                eprintln!("이벤트 핸들 닫기 실패(c2s): {:?}", e);
            }

            if let Err(e) = CloseHandle(self.h_map_file) {
                eprintln!("파일 매핑 핸들 닫기 실패: {:?}", e);
            }
        }
    }
}
