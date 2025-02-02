#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;

use bitflags::bitflags;
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

bitflags! {
    pub struct EoSModeFlags: i32 {
        const REOS = eos_flags_REOS as i32;
        const XEOS = eos_flags_XEOS as i32;
        const BIN = eos_flags_BIN as i32;
    }
}

pub struct EoS {
    flags: EoSModeFlags,
    character: u8,
}

impl EoS {
    pub fn new(flags: EoSModeFlags, character: u8) -> Self {
        EoS { flags, character }
    }

    pub fn new_alt(reos: bool, xeos: bool, bin: bool, character: u8) -> Self {
        let mut flags: EoSModeFlags = EoSModeFlags::empty();

        if reos {
            flags |= EoSModeFlags::REOS;
        }
        if xeos {
            flags |= EoSModeFlags::XEOS;
        }
        if bin {
            flags |= EoSModeFlags::BIN;
        }

        EoS { flags, character }
    }
}

impl Into<i32> for EoS {
    fn into(self) -> i32 {
        self.flags.bits() as i32 | self.character as i32
    }
}

pub struct Device {
    descriptor: i32,
}

impl Device {
    pub fn new(
        board_index: i32,
        pad: i32,
        sad: Option<i32>,
        timo: i32,
        send_eoi: bool,
        eosmode: Option<EoS>,
    ) -> Result<Self, Error> {
        let descriptor;

        let sad = if let Some(s) = sad { s + 0x60 } else { 0 };
        let eosmode = if let Some(mode) = eosmode {
            mode.into()
        } else {
            0
        };

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
    fn id_hp3457() {
        let device = Device::new(
            0,
            22,
            None,
            20,
            true,
            Some(EoS::new(EoSModeFlags::REOS, b'\n')),
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
