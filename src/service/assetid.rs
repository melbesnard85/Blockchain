use exonum::encoding::{CheckedOffset, Field, Offset, Result as ExonumResult};
use exonum::encoding::serialize::WriteBufferWrapper;
use exonum::encoding::serialize::json::ExonumJson;
use serde_json;
use serde_json::value::Value;
use std::error::Error;
use std::mem;
use std::string::ToString;

/// A 128-bit (16 byte) buffer containing the ID.
pub type AssetIDBytes = [u8; 16];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AssetID {
    /// The 128-bit number stored in 16 bytes
    bytes: AssetIDBytes,
}

/// Error details for string parsing failures.
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ParseError {
    InvalidLength(usize),
    InvalidCharacter(char, usize),
    UnexpectedErrorAt(usize),
}

impl AssetID {
    pub fn nil() -> AssetID {
        AssetID { bytes: [0u8; 16] }
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    pub fn from_bytes(b: &[u8]) -> Result<AssetID, ParseError> {
        let len = b.len();
        if len != mem::size_of::<AssetIDBytes>() {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetID::nil();
        assetid.bytes.copy_from_slice(b);
        Ok(assetid)
    }

    pub fn from_str(us: &str) -> Result<AssetID, ParseError> {
        let len = us.len();
        if len != 32 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut cs = us.chars().enumerate();
        for (i, c) in cs.by_ref() {
            if !c.is_digit(16) {
                return Err(ParseError::InvalidCharacter(c, i))
            }
        }

        let mut bytes = [0u8; 16];

        for i in 0..bytes.len() {
            let offset = i * 2;
            let to = offset + 2;
            match u8::from_str_radix(&us[offset..to], 16) {
                Ok(byte) => bytes[i] = byte,
                Err(..) => return Err(ParseError::UnexpectedErrorAt(offset)),
            }
        }

        AssetID::from_bytes(&bytes)
    }
}

impl ToString for AssetID {
    fn to_string(&self) -> String {
        let mut assetid_hex = "".to_string();
        let len = self.bytes.len();
        for i in 0..len {
            let byte_hex = format!("{:02x}", self.bytes[i]);
            assetid_hex += &*byte_hex;
        }
        assetid_hex
    }
}

impl<'a> Field<'a> for AssetID {
    fn field_size() -> Offset {
        mem::size_of::<AssetIDBytes>() as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> AssetID {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&buffer[from as usize..to as usize]);
        AssetID { bytes: bytes }
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, to: Offset) {
        buffer[from as usize..to as usize].copy_from_slice(&self.bytes);
    }

    fn check(
        _: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> ExonumResult {
        debug_assert_eq!((to - from)?.unchecked_offset(), Self::field_size());
        Ok(latest_segment)
    }
}

impl ExonumJson for AssetID {
    fn deserialize_field<B: WriteBufferWrapper>(
        value: &Value,
        buffer: &mut B,
        from: Offset,
        to: Offset,
    ) -> Result<(), Box<Error>> {
        let string: String = serde_json::from_value(value.clone()).unwrap();
        let asset_id = AssetID::from_str(&string);
        // TODO: FIX ME
        if asset_id.is_ok() {
            buffer.write(from, to, asset_id.unwrap());
        }
        Ok(())
    }

    fn serialize_field(&self) -> Result<Value, Box<Error>> {
        let string = self.to_string();
        Ok(Value::String(string))
    }
}


#[cfg(test)]
mod tests {
    use exonum::encoding::{Field, Offset};
    use super::AssetID;
    use super::ParseError::*;

    #[test]
    fn test_nil() {
        let assetid = AssetID::nil();
        let expected = "00000000000000000000000000000000";

        assert_eq!(assetid.to_string(), expected);
        assert_eq!(assetid.as_bytes(), &[0u8; 16]);
    }

    #[test]
    fn test_from_bytes() {
        let b = [0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2,
                 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let assetid = AssetID::from_bytes(&b).unwrap();

        assert_eq!(assetid.as_bytes(), &b);
    }

    #[test]
    fn test_from_str() {
        // Invalid
        assert_eq!(AssetID::from_str(""), Err(InvalidLength(0)));
        assert_eq!(AssetID::from_str("!"), Err(InvalidLength(1)));
        assert_eq!(
            AssetID::from_str("67e5504410b1426%9247bb680e5fe0c8"),
            Err(InvalidCharacter('%', 15))
        );

        // Valid
        assert!(AssetID::from_str("00000000000000000000000000000000").is_ok());
        assert!(AssetID::from_str("67e5504410b1426f9247bb680e5fe0c8").is_ok()); 
    }

    #[test]
    fn test_as_bytes() {
        let expected = [0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2,
                0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
        let assetid = AssetID::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();

        assert_eq!(assetid.as_bytes(), &expected);
    }

    #[test]
    fn test_to_string() {
        let b = [0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2,
                 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];

        let assetid = AssetID::from_bytes(&b).unwrap();
        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";

        assert_eq!(assetid.to_string(), expected);
    }

    #[test]
    fn test_read() {
        let buffer = vec![0;16];
        unsafe {
            let assetid = AssetID::read(&buffer, 0, 16);
            assert_eq!(assetid, AssetID::nil());
        }

        let buffer = vec! [0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2,
                           0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
        unsafe {
            let assetid = AssetID::read(&buffer, 0, buffer.len() as Offset);
            let expected = AssetID::from_bytes(&buffer).unwrap();
            assert_eq!(assetid, expected);
        }

        let mut extended_buffer = vec! [0xde, 0xad];
        extended_buffer.extend(&buffer);
        unsafe {
            let assetid = AssetID::read(&extended_buffer, 2, extended_buffer.len() as Offset);
            let expected = AssetID::from_bytes(&buffer).unwrap();
            assert_eq!(assetid, expected);
        }
    }

    #[test]
    fn test_write() {
        let expected =[0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2,
                             0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8];
        let assetid = AssetID::from_bytes(&expected).unwrap();
        let mut buffer = vec! [0; expected.len()];

        assetid.write(&mut buffer, 0, expected.len() as Offset);
        assert_eq!(buffer, expected);

        let expected =[0x0, 0x0, 0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2,
                             0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0x0, 0x0];
        let assetid = AssetID::from_bytes(&expected[2..18]).unwrap();
        let mut buffer = vec! [0; expected.len()];

        assetid.write(&mut buffer, 2, 18);
        assert_eq!(buffer, expected);
    }
}