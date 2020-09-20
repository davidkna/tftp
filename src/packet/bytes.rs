use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::io::{self, ErrorKind, Result};

use crate::util::FirstNul;

use super::*;

impl TryFrom<u16> for Opcode {
    type Error = io::Error;

    fn try_from(val: u16) -> Result<Opcode> {
        Ok(match val {
            o if o == Opcode::Rrq as u16 => Opcode::Rrq,
            o if o == Opcode::Wrq as u16 => Opcode::Wrq,
            o if o == Opcode::Data as u16 => Opcode::Data,
            o if o == Opcode::Ack as u16 => Opcode::Ack,
            o if o == Opcode::Error as u16 => Opcode::Error,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl From<Opcode> for u16 {
    fn from(op: Opcode) -> u16 {
        op as u16
    }
}

impl TryFrom<String> for Mode {
    type Error = io::Error;

    fn try_from(mut s: String) -> Result<Mode> {
        s.make_ascii_lowercase();

        Ok(match s.as_str() {
            "mail" => Mode::Mail,
            "netascii" => Mode::NetAscii,
            "octet" => Mode::Octet,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl From<Mode> for String {
    fn from(mode: Mode) -> String {
        match mode {
            Mode::Mail => "mail".to_string(),
            Mode::NetAscii => "netascii".to_string(),
            Mode::Octet => "octet".to_string(),
        }
    }
}

impl TryFrom<CString> for Mode {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: CString) -> std::result::Result<Mode, Self::Error> {
        let s = String::from_utf8(s.into_bytes())?;

        Mode::try_from(s).map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
    }
}

impl From<Mode> for CString {
    fn from(mode: Mode) -> CString {
        let s = String::from(mode);

        // This is safe because none of the Mode variants' String
        // representations have a NUL-byte in them.
        unsafe { CString::from_vec_unchecked(s.into_bytes()) }
    }
}

impl From<ErrorCode> for u16 {
    fn from(e: ErrorCode) -> u16 {
        e as u16
    }
}

impl TryFrom<u16> for ErrorCode {
    type Error = io::Error;

    fn try_from(val: u16) -> Result<ErrorCode> {
        Ok(match val {
            e if e == ErrorCode::NotDefined as u16 => ErrorCode::NotDefined,
            e if e == ErrorCode::FileNotFound as u16 => ErrorCode::FileNotFound,
            e if e == ErrorCode::AccessViolation as u16 => ErrorCode::AccessViolation,
            e if e == ErrorCode::DiskFull as u16 => ErrorCode::DiskFull,
            e if e == ErrorCode::IllegalOperation as u16 => ErrorCode::IllegalOperation,
            e if e == ErrorCode::UnknownTid as u16 => ErrorCode::UnknownTid,
            e if e == ErrorCode::FileAlreadyExists as u16 => ErrorCode::FileAlreadyExists,
            e if e == ErrorCode::NoSuchUser as u16 => ErrorCode::NoSuchUser,
            _ => return Err(ErrorKind::InvalidInput.into()),
        })
    }
}

impl TryFrom<Vec<u8>> for Rq {
    type Error = io::Error;

    fn try_from(mut bytes: Vec<u8>) -> Result<Rq> {
        let nul = match bytes.find_first_nul() {
            Some(n) => n,
            None => return Err(ErrorKind::InvalidInput.into()),
        };

        /* splitting off 1-byte past the NUL byte because we don't want
         * to include the NUL byte in the returned Vec */
        let split_at = nul + 1;

        /* split_off panics if at > len */
        if split_at > bytes.len() {
            return Err(ErrorKind::InvalidInput.into());
        }

        let mut mode = bytes.split_off(nul + 1);

        /* drop the nul terminators, these will be added by CString::new
         * and CString::new fails when they are already present. */
        let _ = mode.pop();
        let _ = bytes.pop();

        let mode = CString::new(mode)
            .map(|c| -> std::result::Result<Mode, Box<dyn std::error::Error>> { c.try_into() })?
            .map_err(|_| -> io::Error { ErrorKind::InvalidInput.into() })?;
        let filename = CString::new(bytes)
            .map_err(|_| -> io::Error { ErrorKind::InvalidInput.into() })?
            .into_string()
            .map_err(|_| -> io::Error { ErrorKind::InvalidInput.into() })?;

        Ok(Rq { filename, mode })
    }
}

impl From<Rq> for Vec<u8> {
    fn from(rq: Rq) -> Vec<u8> {
        let mode: String = rq.mode.into();

        let mut bytes = vec![];
        bytes.append(&mut rq.filename.into_bytes());
        bytes.append(&mut vec![0]);
        bytes.append(&mut mode.into_bytes());
        bytes.append(&mut vec![0]);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversions() {
        assert_eq!(u16::from(Opcode::Rrq), 1);
        assert_eq!(u16::from(Opcode::Wrq), 2);
        assert_eq!(u16::from(Opcode::Data), 3);
        assert_eq!(u16::from(Opcode::Ack), 4);
        assert_eq!(u16::from(Opcode::Error), 5);

        assert!(Opcode::try_from(0).is_err());
        assert_eq!(Opcode::Rrq, Opcode::try_from(1).unwrap());
        assert_eq!(Opcode::Wrq, Opcode::try_from(2).unwrap());
        assert_eq!(Opcode::Data, Opcode::try_from(3).unwrap());
        assert_eq!(Opcode::Ack, Opcode::try_from(4).unwrap());
        assert_eq!(Opcode::Error, Opcode::try_from(5).unwrap());
        assert!(Opcode::try_from(6).is_err());
        assert!(Opcode::try_from(12).is_err());
    }

    #[test]
    fn test_mode_conversions() {
        assert_eq!("mail", &String::from(Mode::Mail));
        assert_eq!("netascii", &String::from(Mode::NetAscii));
        assert_eq!("octet", &String::from(Mode::Octet));
        assert_eq!(Mode::Mail, Mode::try_from("mail".to_string()).unwrap());
        assert_eq!(Mode::NetAscii, Mode::try_from("netascii".to_string()).unwrap());
        assert_eq!(Mode::Octet, Mode::try_from("octet".to_string()).unwrap());
        assert_eq!(Mode::Mail, Mode::try_from(CString::new("mail").unwrap()).unwrap());
        assert_eq!(Mode::NetAscii, Mode::try_from(CString::new("netascii").unwrap()).unwrap());
        assert_eq!(Mode::Octet, Mode::try_from(CString::new("octet").unwrap()).unwrap());
        assert_eq!(Mode::Mail, Mode::try_from("MaIL".to_string()).unwrap());
        assert_eq!(Mode::NetAscii, Mode::try_from("NETASCII".to_string()).unwrap());
        assert_eq!(Mode::Octet, Mode::try_from("OCtet".to_string()).unwrap());
        assert!(Mode::try_from("PotAtOO".to_string()).is_err());
    }

    #[test]
    fn test_errorcode_conversions() {
        assert_eq!(u16::from(ErrorCode::NotDefined), 0);
        assert_eq!(u16::from(ErrorCode::FileNotFound), 1);
        assert_eq!(u16::from(ErrorCode::AccessViolation), 2);
        assert_eq!(u16::from(ErrorCode::DiskFull), 3);
        assert_eq!(u16::from(ErrorCode::IllegalOperation), 4);
        assert_eq!(u16::from(ErrorCode::UnknownTid), 5);
        assert_eq!(u16::from(ErrorCode::FileAlreadyExists), 6);
        assert_eq!(u16::from(ErrorCode::NoSuchUser), 7);

        assert!(ErrorCode::try_from(8).is_err());
        assert_eq!(ErrorCode::NotDefined, ErrorCode::try_from(0).unwrap());
        assert_eq!(ErrorCode::FileNotFound, ErrorCode::try_from(1).unwrap());
        assert_eq!(ErrorCode::AccessViolation, ErrorCode::try_from(2).unwrap());
        assert_eq!(ErrorCode::DiskFull, ErrorCode::try_from(3).unwrap());
        assert_eq!(ErrorCode::IllegalOperation, ErrorCode::try_from(4).unwrap());
        assert_eq!(ErrorCode::UnknownTid, ErrorCode::try_from(5).unwrap());
        assert_eq!(ErrorCode::FileAlreadyExists, ErrorCode::try_from(6).unwrap());
        assert_eq!(ErrorCode::NoSuchUser, ErrorCode::try_from(7).unwrap());
    }

    #[test]
    fn test_rq_from_bytes() {
        let bytes = vec![b'h', b'i', b'.', b't', b'x', b't', b'\0', b'n', b'e', b't', b'a', b's', b'c', b'i', b'i', b'\0'];
        let rq = Rq::try_from(bytes).unwrap();

        assert_eq!(rq.filename, "hi.txt".to_string());
        assert_eq!(rq.mode, Mode::NetAscii);
    }

    #[test]
    fn test_rq_to_bytes() {
        let rq = Rq {
            filename: "bye.txt".to_string(),
            mode: Mode::Mail,
        };

        let bytes: Vec<u8> = rq.into();
        assert_eq!(bytes, vec![b'b', b'y', b'e', b'.', b't', b'x', b't', b'\0', b'm', b'a', b'i', b'l', b'\0']);
    }
}
