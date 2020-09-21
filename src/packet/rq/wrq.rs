use std::io::{self, Result};

use crate::bytes::{FromBytes, IntoBytes};
use crate::packet::mode::Mode;
use crate::packet::opcode::Opcode;
use crate::packet::sealed::Packet;
use super::Rq;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wrq(Rq);

impl Wrq {
    pub fn new<T: AsRef<str>>(filename: T, mode: Mode) -> Self {
        let filename = filename.as_ref().to_string();
        Self(Rq { filename, mode })
    }
}

impl Packet for Wrq {
    const OPCODE: Opcode = Opcode::Wrq;
}

impl FromBytes for Wrq {
    type Error = io::Error;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let rq = Rq::from_bytes(bytes)?;

        Ok(Self(rq))
    }
}

impl IntoBytes for Wrq {
    fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}
