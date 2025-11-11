#![allow(dead_code)]

use crate::api::device::{Device, TpxDevice};
use crate::api::ffi::*;
use std::ffi::CString;

/// Only one PixHandle should ever exist
///
/// Cannot be a static, because those dont run Drop destructors
pub struct PixHandle {}

impl PixHandle {
    pub fn new() -> Self {
        unsafe {
            let conf_dir = CString::new("config").unwrap();
            let log_dir = CString::new("log").unwrap();
            pxcSetDirectories(conf_dir.as_ptr(), log_dir.as_ptr());

            pxcInitialize(0, std::ptr::null());
        }

        Self {}
    }
}

use crate::api::device::TpxMode;
impl PixHandle {
    /// Returns amount of currently connected devices
    ///
    /// [out] amount of devices
    pub fn get_device_count(&self) -> i32 {
        unsafe { pxcGetDevicesCount() }
    }

    /// Removes disconnected devices. Searches for new ones.
    ///
    /// [out] undefined return, most probably status
    pub fn refresh_devices(&self) -> i32 {
        unsafe { pxcRefreshDevices() }
    }

    /// Builds device from a `DeviceBuilder`
    pub fn get_device(&self, builder: DeviceBuilder) -> PxcResult<impl Device> {
        match builder.info.r#type {
            DevType::Tpx => {
                let mut width: std::ffi::c_uint = 0;
                let mut height: std::ffi::c_uint = 0;
                unsafe {
                    pxcSetTimepixMode(builder.index, TpxMode::Tot as i32).check_rc()?;
                    pxcSetTimepixCalibrationEnabled(builder.index, true).check_rc()?;
                    pxcGetDeviceDimensions(builder.index, &mut width, &mut height).check_rc()?;
                }
                let device = TpxDevice {
                    index: builder.index,
                    frame_time: builder.frame_time.unwrap_or(2.0),
                    dimensions: (width, height),
                };
                device.set_high_voltage(builder.high_voltage.unwrap_or(40.0))?;
                device.set_threshold(builder.threshold.unwrap_or(200.0))?;
                Ok(device)
            }
            _ => unimplemented!(),
        }
    }
}

impl Drop for PixHandle {
    fn drop(&mut self) {
        unsafe {
            pxcExit();
        }
    }
}

#[derive(Default)]
pub struct DeviceBuilder {
    index: std::ffi::c_uint,
    info: CDevInfo,
    frame_time: Option<std::ffi::c_double>,
    high_voltage: Option<std::ffi::c_double>,
    threshold: Option<std::ffi::c_double>,
}

impl DeviceBuilder {
    pub fn new(index: u32) -> Self {
        let mut info = CDevInfo::default();
        unsafe {
            pxcGetDeviceInfo(index, &mut info);
        }

        Self {
            index,
            info,
            ..DeviceBuilder::default()
        }
    }

    /// time to capture a frame for in seconds
    pub fn frame_time(mut self, seconds: f64) -> Self {
        self.frame_time = Some(seconds);
        self
    }
    /// voltage with which to collect the charges from the pixels
    /// - `voltage`: safe values are 5V - 100V
    ///
    /// defaults to 40V if not set
    pub fn high_voltage(mut self, voltage: f64) -> Self {
        self.high_voltage = Some(voltage);
        self
    }
    /// values 100 - 500
    ///
    /// defaults to 200 if not set
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

#[repr(C, packed)]
pub struct CDevInfo {
    name: [std::ffi::c_char; 20],
    serial: u32,
    r#type: DevType,
}

impl Default for CDevInfo {
    fn default() -> Self {
        Self {
            name: [0 as std::ffi::c_char; 20],
            serial: 0,
            r#type: DevType::Tpx,
        }
    }
}

#[repr(C)]
enum DevType {
    Tpx = 1,
    Mpx3,
    Tpx3,
    Tpx2,
}
