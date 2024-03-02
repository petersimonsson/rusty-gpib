#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;

use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create device descriptor")]
    CreateFailed(i32),
    #[error("Failed to write to device")]
    WriteFailed(i32),
    #[error("Failed to read from device")]
    ReadFailed(i32),
}

pub struct Device {
    descriptor: i32,
}

impl Device {
    pub fn new(
        board_index: i32,
        pad: i32,
        sad: i32,
        timo: i32,
        send_eoi: i32,
        eosmode: i32,
    ) -> Result<Self, Error> {
        let descriptor;

        unsafe {
            descriptor = ibdev(board_index, pad, sad, timo, send_eoi, eosmode);
        }

        if descriptor == -1 {
            let error;
            unsafe {
                error = ThreadIberr();
            }
            return Err(Error::CreateFailed(error));
        }

        Ok(Device { descriptor })
    }

    pub fn write(&self, data: &[u8]) -> Result<(), Error> {
        let status;
        unsafe {
            status = ibwrt(
                self.descriptor,
                data as *const _ as *const c_void,
                data.len() as i64,
            );
        }

        if status & 0x8000 != 0 {
            let error;
            unsafe {
                error = ThreadIberr();
            }
            return Err(Error::WriteFailed(error));
        }

        Ok(())
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<i32, Error> {
        let status;
        unsafe {
            status = ibrd(
                self.descriptor,
                buf as *mut _ as *mut c_void,
                buf.len() as i64,
            );
        }

        if status & 0x8000 != 0 {
            let error;
            unsafe {
                error = ThreadIberr();
            }
            Err(Error::ReadFailed(error))
        } else {
            let count;
            unsafe {
                count = ThreadIbcnt();
            }
            Ok(count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_dcv_hp3457() {
        let device = Device::new(0, 22, 0, 10, 1, 0).unwrap();
        let status = device.write(b"DCV");

        assert!(status.is_ok());
    }
}
