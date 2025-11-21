#![allow(dead_code)]

use std::ffi::CString;

use crate::api::ffi::*;
use crate::api::ffi::{PxcBuffer, PxcResult};
use std::ffi::{c_double, c_uint};

pub trait Device: Send + Sync {
    fn capture_image(&self) -> PxcResult<PxcBuffer>;
    fn save_last_frame(&self, file_path: &str) -> PxcResult<()>;
    fn get_dimensions(&self) -> (c_uint, c_uint);

    fn get_voltage_range(&self) -> PxcResult<(c_double, c_double)>;
    fn set_high_voltage(&self, voltage: c_double) -> PxcResult<()>;
    fn set_threshold(&self, threshold: c_double) -> PxcResult<()>;
    fn set_frame_time(&mut self, seconds: c_double) -> PxcResult<()>;

    fn set_software_high_threshold(&mut self, high_threshold: f64);
    fn set_software_low_threshold(&mut self, low_threshold: f64);
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
    pub low_threshold: f64,
    pub high_threshold: f64,
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
        if self.high_threshold != 0.0 {
            for val in data_buf.iter_mut() {
                if *val > self.high_threshold as std::ffi::c_short {
                    *val = 0;
                }
            }
        }
        for val in data_buf.iter_mut() {
            if *val < self.low_threshold as std::ffi::c_short {
                *val = 0;
            }
        }
        Ok(data_buf)
    }

    fn save_last_frame(&self, file_path: &str) -> PxcResult<()> {
        unsafe {
            let c_file_path: CString = CString::new(file_path).unwrap();
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
            pxcSetBias(self.index, voltage).check_rc()?;
        }
        Ok(())
    }

    fn set_software_high_threshold(&mut self, high_threshold: f64) {
        self.high_threshold = high_threshold;
    }
    fn set_software_low_threshold(&mut self, low_threshold: f64) {
        self.low_threshold = low_threshold;
    }
}
