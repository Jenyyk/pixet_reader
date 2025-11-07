#![allow(dead_code)]

use std::ffi::CString;

use crate::api::ffi::*;
use crate::api::ffi::{PxcBuffer, PxcResult};

pub trait Device {
    fn capture_image(&self) -> PxcResult<PxcBuffer>;
    fn save_last_frame(&self, file_path: impl Into<String>) -> PxcResult<()>;
    fn get_dimensions(&self) -> (std::ffi::c_uint, std::ffi::c_uint);
}

pub enum TpxMode {
    /// counting mode
    Medipix = 0,
    /// energy mode
    Tot = 1,
    /// timepix mode
    Timepix = 3,
}

pub struct TpxDevice {
    pub index: std::ffi::c_uint,
    pub frame_time: std::ffi::c_double,
    pub dimensions: (std::ffi::c_uint, std::ffi::c_uint),
}

impl Device for TpxDevice {
    fn capture_image(&self) -> PxcResult<PxcBuffer> {
        let mut data_buf: PxcBuffer = [0; 65536];
        let mut size: std::ffi::c_uint = 65536;
        unsafe {
            pxcMeasureSingleFrame(self.index, self.frame_time, &mut data_buf, &mut size)
                .check_rc()?;

            if size == 0 {
                size = 65536;
                pxcGetMeasuredFrame(self.index, 0, &mut data_buf, &mut size).check_rc()?;
            }
        }
        Ok(data_buf)
    }

    fn save_last_frame(&self, file_path: impl Into<String>) -> PxcResult<()> {
        unsafe {
            let c_file_path: CString = CString::new(file_path.into()).unwrap();
            pxcSaveMeasuredFrame(self.index, 0, c_file_path.as_ptr()).check_rc()?;
        }
        Ok(())
    }

    fn get_dimensions(&self) -> (std::ffi::c_uint, std::ffi::c_uint) {
        self.dimensions
    }
}
