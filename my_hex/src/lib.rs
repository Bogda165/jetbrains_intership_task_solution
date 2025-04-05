#[derive(Debug, Clone)]
pub enum HexError {
    OutOfBounds,
}

//simple trait to allow encoding data in hex
pub trait ToHex {
    fn to_hex(val: u8) -> Result<Self, HexError>
    where
        Self: Sized;

    fn encode<I: Iterator<Item = u8>, R: FromIterator<Self>>(data: I) -> R
    where
        Self: Sized;
}

impl ToHex for char {
    fn to_hex(val: u8) -> Result<Self, HexError> {
        Ok(match val {
            0..=9 => (b'0' + val) as char,
            10..16 => (b'a' + val - 10) as char,
            _ => return Err(HexError::OutOfBounds),
        })
    }

    fn encode<I: Iterator<Item = u8>, R: FromIterator<Self>>(data: I) -> R {
        data.into_iter()
            .flat_map(|byte| {
                let high = byte >> 4;
                let low = byte & 0x0F;
                [Self::to_hex(high).unwrap(), Self::to_hex(low).unwrap()]
            })
            .collect()
    }
}

mod tests {

    #[test]
    fn test_single_byte() {
        let data = [0x4F];
        let hex: String = char::encode(data.iter().copied());
        assert_eq!(hex, "4f");
    }

    #[test]
    fn test_multiple_bytes() {
        let data = [0xAB, 0xCD, 0xEF];
        let hex: String = char::encode(data.iter().copied());
        assert_eq!(hex, "abcdef");
    }

    #[test]
    fn test_empty() {
        let data: [u8; 0] = [];
        let hex: String = char::encode(data.iter().copied());
        assert_eq!(hex, "");
    }

    #[test]
    fn test_all_nibbles() {
        let data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let hex: String = char::encode(data.iter().copied());
        assert_eq!(hex, "0123456789abcdef");
    }
}
