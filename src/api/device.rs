#![allow(dead_code)]

use std::ffi::CString;

use crate::api::ffi::*;
use crate::api::ffi::{PxcBuffer, PxcResult};
use std::ffi::{c_uint, c_double};

pub trait Device {
    fn capture_image(&self) -> PxcResult<PxcBuffer>;
    fn save_last_frame(&self, file_path: impl Into<String>) -> PxcResult<()>;
    fn get_dimensions(&self) -> (c_uint, c_uint);

    fn get_voltage_range(&self) -> PxcResult<(c_double, c_double)>;
    fn set_high_voltage(&self, voltage: c_double) -> PxcResult<()>;
    fn set_threshold(&self, threshold: c_double) -> PxcResult<()>;
    fn set_frame_time(&mut self, seconds: c_double) -> PxcResult<()>;
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

    fn set_threshold(&self, threshold: c_double) -> PxcResult<()> {
        unsafe {
            pxcSetThreshold(self.index, 0, threshold).check_rc()?;
        }
        Ok(())
    }

    fn set_frame_time(&mut self, seconds: c_double) -> PxcResult<()> {
        self.frame_time = seconds;
        Ok(())
    }

    fn get_voltage_range(&self) -> PxcResult<(c_double, c_double)> {
        let mut min_voltage = 0.0;
        let mut max_voltage = 0.0;
        unsafe {
            pxcGetBiasRange(self.index, &mut min_voltage, &mut max_voltage).check_rc()?;
        }
        Ok((min_voltage, max_voltage))
    }
    fn set_high_voltage(&self, voltage: c_double) -> PxcResult<()> {
        unsafe {
            pxcSetBias(self.index, voltage);
        }
        Ok(())
    }
}
