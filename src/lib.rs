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

pub enum Eoi {
    None,
    AssertOnSend,
}

impl From<Eoi> for i32 {
    fn from(value: Eoi) -> Self {
        match value {
            Eoi::None => 0,
            Eoi::AssertOnSend => 1,
        }
    }
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
        send_eoi: Eoi,
        eosmode: i32,
    ) -> Result<Self, Error> {
        let descriptor;

        unsafe {
            descriptor = ibdev(board_index, pad, sad, timo, send_eoi.into(), eosmode);
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
        let device = Device::new(
            0,
            22,
            0,
            20,
            Eoi::AssertOnSend,
            eos_flags_REOS as i32 | '\n' as i32,
        )
        .unwrap();

        let status = device.write(b"ID?");
        assert!(status.is_ok());

        let mut data: [u8; 20] = [0; 20];

        match device.read(&mut data) {
            Ok(len) => println!("{} => {:?}", len, String::from_utf8(data.to_vec())),
            Err(e) => println!("{:?}", e),
        }
    }
}
