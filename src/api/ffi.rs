#![allow(dead_code)]

use std::ffi::{c_char, c_double, c_int, c_uint};

type CStringPointer = *const c_char;

pub type PxcBuffer = [std::ffi::c_short; 65536];

pub type PxcResult<T> = Result<T, PxcErr>;

#[derive(Debug)]
pub enum PxcErr {
    NotInitialized = -1,
    InvalidDeviceIndex = -2,
    InvalidArgument = -3,
    CouldNotSave = -4,
    AcqFailed = -5,
    DeviceError = -6,
    AcqAborted = -7,
    CannotReconnect = -8,
    NotAllowed = -9,
    NotSupported = -10,
    BufferSmall = -11,
    CannotCalibrate = -12,
    TooManyBadPixels = -13,
    ZestNotLoaded = -14,
    UnexpectedError = -1000,
}
impl From<i32> for PxcErr {
    fn from(val: i32) -> Self {
        match val {
            -1 => PxcErr::NotInitialized,
            -2 => PxcErr::InvalidDeviceIndex,
            -3 => PxcErr::InvalidArgument,
            -4 => PxcErr::CouldNotSave,
            -5 => PxcErr::AcqFailed,
            -6 => PxcErr::DeviceError,
            -7 => PxcErr::AcqAborted,
            -8 => PxcErr::CannotReconnect,
            -9 => PxcErr::NotAllowed,
            -10 => PxcErr::NotSupported,
            -11 => PxcErr::BufferSmall,
            -12 => PxcErr::CannotCalibrate,
            -13 => PxcErr::TooManyBadPixels,
            -14 => PxcErr::ZestNotLoaded,
            _ => PxcErr::UnexpectedError,
        }
    }
}

/// helper trait to check the return code of FFI functions
pub trait PxcErrCheck {
    fn check_rc(self) -> PxcResult<()>;
}
impl PxcErrCheck for c_int {
    /// helper function to convert the return code of FFI functions into a `Result`
    fn check_rc(self) -> PxcResult<()> {
        if self < 0 {
            return Err(PxcErr::from(self));
        }
        Ok(())
    }
}

#[link(name = "pxcore", kind = "dylib")]
unsafe extern "C" {
    pub fn pxcSetDirectories(config_dir: CStringPointer, log_dir: CStringPointer) -> c_int;
    pub fn pxcInitialize(argc: c_int, argv: *const *const c_char) -> c_int;

    pub fn pxcGetDevicesCount() -> c_int;
    pub fn pxcRefreshDevices() -> c_int;

    pub fn pxcGetDeviceName(index: c_uint, nameBuffer: *mut c_char, size: c_uint) -> c_int;
    pub fn pxcGetDeviceDimensions(index: c_uint, width: *mut c_uint, height: *mut c_uint) -> c_int;
    pub fn pxcGetDeviceInfo(index: c_uint, devInfo: *mut crate::api::handle::CDevInfo) -> c_int;

    pub fn pxcLoadFactoryConfig(index: c_uint) -> c_int;

    pub fn pxcGetBiasRange(index: c_uint, minBias: *mut c_double, maxBias: *mut c_double) -> c_int;
    pub fn pxcSetBias(index: c_uint, bias: c_double) -> c_int;
    pub fn pxcSetThreshold(index: c_uint, thresholdIndex: c_int, threshold: c_double) -> c_int;
    pub fn pxcSetTimepixMode(index: c_uint, mode: c_int) -> c_int;

    pub fn pxcGetMeasuredFrameCount(index: c_uint) -> c_int;
    pub fn pxcMeasureSingleFrame(
        index: c_uint,
        frameTime: std::ffi::c_double,
        frameData: &mut PxcBuffer,
        size: &mut c_uint,
    ) -> c_int;
    pub fn pxcGetMeasuredFrame(
        index: c_uint,
        frameIndex: c_uint,
        frameData: &mut PxcBuffer,
        size: &mut c_uint,
    ) -> c_int;

    pub fn pxcSaveMeasuredFrame(
        index: c_uint,
        frameIndex: c_uint,
        filePath: CStringPointer,
    ) -> c_int;

    pub fn pxcExit() -> c_int;
}
