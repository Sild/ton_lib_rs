pub mod ser_de;

use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::TonlibError;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::fmt::{Debug, Display, UpperHex};
use std::hash::Hash;
use std::str::FromStr;

#[derive(Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub struct TonHash(TonHashData);

#[derive(Debug, PartialOrd, Ord, Clone)]
enum TonHashData {
    Slice([u8; 32]),
    Vec(Vec<u8>),
}

impl TonHash {
    pub const BYTES_LEN: usize = 32;
    pub const BITS_LEN: usize = 256;
    pub const ZERO: TonHash = TonHash::from_slice(&[0u8; 32]);
    pub const EMPTY_CELL_HASH: TonHash = TonHash::from_slice(&[
        150, 162, 150, 210, 36, 242, 133, 198, 123, 238, 147, 195, 15, 138, 48, 145, 87, 240, 218, 163, 93, 197, 184,
        126, 65, 11, 120, 99, 10, 9, 207, 199,
    ]);

    pub const fn from_slice(data: &[u8; 32]) -> Self { Self(TonHashData::Slice(*data)) }

    pub fn from_bytes(data: &[u8]) -> Result<Self, TonlibError> {
        check_bytes_len(data)?;
        Ok(Self::from_slice(data[..32].try_into().unwrap()))
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, TonlibError> {
        check_bytes_len(&data)?;
        Ok(Self(TonHashData::Vec(data)))
    }

    pub fn from_num<T: TonCellNum>(num: &T) -> Result<Self, TonlibError> {
        if T::IS_PRIMITIVE {
            return Err(TonlibError::TonHashWrongLen {
                exp: TonHash::BYTES_LEN,
                given: 128, // max primitive size
            });
        }
        Self::from_bytes(&num.tcn_to_bytes())
    }

    pub fn as_slice(&self) -> &[u8] { self.0.as_slice() }

    pub fn as_slice_sized(&self) -> &[u8; 32] {
        match &self.0 {
            TonHashData::Slice(data) => data,
            TonHashData::Vec(data) => data.as_slice().try_into().unwrap(),
        }
    }

    pub fn as_mut_slice_sized(&mut self) -> &mut [u8; 32] {
        match &mut self.0 {
            TonHashData::Slice(data) => data,
            TonHashData::Vec(data) => data.as_mut_slice().try_into().unwrap(),
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        match &mut self.0 {
            TonHashData::Slice(data) => data,
            TonHashData::Vec(data) => data.as_mut_slice(),
        }
    }

    pub fn to_hex(&self) -> String { hex::encode(self.as_slice()) }
    pub fn to_b64(&self) -> String { BASE64_STANDARD.encode(self.as_slice()) }

    pub fn into_vec(self) -> Vec<u8> {
        match self.0 {
            TonHashData::Slice(data) => data.to_vec(),
            TonHashData::Vec(data) => data,
        }
    }
}

impl FromStr for TonHash {
    type Err = TonlibError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 64 {
            return from_hex(s);
        }
        from_b64(s)
    }
}

impl TonHashData {
    fn as_slice(&self) -> &[u8] {
        match self {
            TonHashData::Slice(data) => data.as_slice(),
            TonHashData::Vec(data) => data.as_slice(),
        }
    }
}

fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<TonHash, TonlibError> {
    let bytes = hex::decode(hex)?;
    check_bytes_len(&bytes)?;
    Ok(TonHash(TonHashData::Vec(bytes)))
}

fn from_b64<T: AsRef<[u8]>>(b64: T) -> Result<TonHash, TonlibError> { TonHash::from_vec(BASE64_STANDARD.decode(b64)?) }

fn check_bytes_len(bytes: &[u8]) -> Result<(), TonlibError> {
    if bytes.len() != TonHash::BYTES_LEN {
        return Err(TonlibError::TonHashWrongLen {
            exp: TonHash::BYTES_LEN,
            given: bytes.len(),
        });
    }
    Ok(())
}

// Must implement it manually, because we don't distinguish between Vec and Slice
impl PartialEq for TonHashData {
    fn eq(&self, other: &Self) -> bool { self.as_slice() == other.as_slice() }
}

impl Eq for TonHashData {}

impl Hash for TonHashData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { state.write(self.as_slice()); }
}

impl From<[u8; 32]> for TonHash {
    fn from(data: [u8; 32]) -> Self { Self(TonHashData::Slice(data)) }
}

impl AsRef<[u8]> for TonHash {
    fn as_ref(&self) -> &[u8] { self.as_slice() }
}

impl UpperHex for TonHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.to_hex().to_uppercase()) }
}

impl Display for TonHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:X}") }
}

impl Debug for TonHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "TonHash[{self:X}]") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_hash_display() -> anyhow::Result<()> {
        let data = [255u8; 32];
        let hash = TonHash::from(data);
        assert_eq!(format!("{}", hash), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        assert_eq!(format!("{:X}", hash), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_slice() -> anyhow::Result<()> {
        let data = [1u8; 32];
        let hash = TonHash::from(data);
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_bytes() -> anyhow::Result<()> {
        let data = [1u8; 32];
        let hash = TonHash::from_bytes(&data)?;
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_vec() -> anyhow::Result<()> {
        let data = [1u8; 32];
        let hash = TonHash::from_bytes(&data)?;
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_hex() -> anyhow::Result<()> {
        let data = [255u8; 32];
        let hash = TonHash::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")?;
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }
}
