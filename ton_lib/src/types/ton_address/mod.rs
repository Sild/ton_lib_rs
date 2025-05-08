pub mod ser_de;

use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCellRef;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use crate::errors::TonlibError::TonAddressParseError;
use crate::types::tlb::block_tlb::msg_address::MsgAddressInt::{Std, Var};
use crate::types::tlb::block_tlb::msg_address::{
    MsgAddress, MsgAddressExt, MsgAddressInt, MsgAddressIntStd, MsgAddressIntVar, MsgAddressNone,
};
use crate::types::tlb::block_tlb::state_init::StateInit;
use crate::types::tlb::tlb_type::TLBType;
use crate::utils::rewrite_bits;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use crc::Crc;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::str::FromStr;

const CRC_16_XMODEM: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TonAddress {
    pub wc: i32,
    pub hash: TonHash,
}

impl TonAddress {
    pub const ZERO: Self = TonAddress::new(0, TonHash::ZERO);

    pub const fn new(wc: i32, hash: TonHash) -> Self { Self { wc, hash } }

    pub fn derive(wc: i32, code: TonCellRef, data: TonCellRef) -> Result<TonAddress, TonlibError> {
        let state_init = StateInit::new(code, data);
        Ok(TonAddress::new(wc, state_init.cell_hash()?))
    }

    pub fn from_msg_address<T: Into<MsgAddress>>(msg_address: T) -> Result<Self, TonlibError> {
        match msg_address.into() {
            MsgAddress::Ext(MsgAddressExt::None(_)) => Ok(TonAddress::ZERO),
            MsgAddress::Int(int) => from_msg_address_int(&int),
            other => {
                raise_address_error(&format!("{other:?}"), "can't make TonAddress from specified MsgAddress")?;
                unreachable!()
            }
        }
    }

    pub fn to_hex(&self) -> String { format!("{}:{}", self.wc, hex::encode(self.hash.as_slice())) }

    pub fn to_b64(&self, mainnet: bool, bounce: bool, urlsafe: bool) -> String {
        let mut buf = [0; 36];
        let tag: u8 = match (mainnet, bounce) {
            (true, true) => 0x11,
            (true, false) => 0x51,
            (false, true) => 0x91,
            (false, false) => 0xD1,
        };
        buf[0] = tag;
        buf[1] = (self.wc & 0xff) as u8;
        buf[2..34].clone_from_slice(self.hash.as_slice());
        let crc = CRC_16_XMODEM.checksum(&buf[0..34]);
        buf[34] = ((crc >> 8) & 0xff) as u8;
        buf[35] = (crc & 0xff) as u8;
        if urlsafe {
            URL_SAFE_NO_PAD.encode(buf)
        } else {
            STANDARD.encode(buf)
        }
    }

    pub fn to_msg_address_none(&self) -> Result<MsgAddressNone, TonlibError> {
        if self != &TonAddress::ZERO {
            return Err(TonlibError::CustomError(format!("Can't convert non-zero address={self} to MsgAddressNone")));
        }
        Ok(MsgAddressNone {})
    }

    pub fn to_msg_address_int(&self) -> MsgAddressInt {
        MsgAddressIntStd {
            anycast: None,
            workchain: self.wc as i8,
            address: self.hash.clone(),
        }
        .into()
    }
}

impl FromStr for TonAddress {
    type Err = TonlibError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 48 {
            return from_b64(s);
        }
        from_hex(s)
    }
}

impl TLBType for TonAddress {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        TonAddress::from_msg_address(MsgAddress::read(parser)?)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> {
        match self.to_msg_address_none() {
            Ok(none) => none.write(builder),
            Err(_) => self.to_msg_address_int().write(builder),
        }
    }
}

impl Display for TonAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.to_b64(true, true, true)) }
}

impl Debug for TonAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("TonAddress(\"{self}\")")).finish()
    }
}

impl PartialOrd for TonAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        let self_hash = self.to_cell().and_then(|c| c.cell_hash());
        let other_hash = other.to_cell().and_then(|c| c.cell_hash());
        match (self_hash, other_hash) {
            (Ok(hash0), Ok(hash1)) => Some(hash0.cmp(&hash1)),
            _ => {
                log::error!("Failed to build cell for addresses: {self:?} and {other:?}");
                None
            }
        }
    }
}

fn from_b64<T: AsRef<str>>(addr: T) -> Result<TonAddress, TonlibError> {
    let addr_str = addr.as_ref();
    if addr_str.chars().any(|c| c == '-' || c == '_') {
        from_bytes(&URL_SAFE_NO_PAD.decode(addr_str)?, addr_str)
    } else {
        from_bytes(&STANDARD.decode(addr_str)?, addr_str)
    }
}

fn from_hex<T: AsRef<str>>(addr: T) -> Result<TonAddress, TonlibError> {
    let addr_str = addr.as_ref();
    let parts: Vec<&str> = addr_str.split(':').collect();

    if parts.len() != 2 {
        raise_address_error(addr_str, "expecting 2 parts divided by ':'")?;
    }

    let wc = parts[0].parse::<i32>()?;

    let hash = TonHash::from_vec(hex::decode(parts[1])?)?;
    Ok(TonAddress::new(wc, hash))
}

fn from_bytes(bytes: &[u8], addr_str: &str) -> Result<TonAddress, TonlibError> {
    if bytes.len() != 36 {
        raise_address_error(addr_str, format!("expecting 36 bytes, got {}", bytes.len()))?;
    }

    let addr_crc = ((bytes[34] as u16) << 8) | bytes[35] as u16;
    if addr_crc != CRC_16_XMODEM.checksum(&bytes[0..34]) {
        raise_address_error(addr_str, "crc32 mismatch")?;
    }

    let address = TonAddress {
        wc: bytes[1] as i8 as i32,
        hash: TonHash::from_bytes(&bytes[2..34])?,
    };
    Ok(address)
}

fn from_msg_address_int(msg_address: &MsgAddressInt) -> Result<TonAddress, TonlibError> {
    let (wc, addr, bits_len, anycast) = match msg_address {
        Std(MsgAddressIntStd {
            workchain,
            address,
            anycast,
        }) => (*workchain as i32, address.as_slice(), 256, anycast.as_ref()),
        Var(MsgAddressIntVar {
            workchain,
            address,
            addr_bits_len,
            anycast,
        }) => (*workchain, address.as_slice(), *addr_bits_len, anycast.as_ref()),
    };

    let anycast = match anycast {
        Some(anycast) => anycast,
        None => return Ok(TonAddress::new(wc, TonHash::from_bytes(addr)?)),
    };

    if bits_len < anycast.rewrite_pfx.len as u32 {
        let err_msg = format!("rewrite_pfx has {} bits, but address has only {bits_len} bits", anycast.rewrite_pfx.len);
        let ext_addr_str = format!("address: {msg_address:?}, anycast: {:?}", anycast);
        raise_address_error(&ext_addr_str, err_msg)?
    }

    let new_prefix = anycast.rewrite_pfx.as_slice();

    let bits = anycast.rewrite_pfx.len;
    let mut addr_mutable = addr.to_vec();

    if !rewrite_bits(new_prefix, 0, addr_mutable.as_mut_slice(), 0, bits) {
        let err_msg = format!(
            "Failed to rewrite address prefix with new_prefix={new_prefix:?}, address={msg_address:?}, bits={bits}"
        );
        let ext_addr_str = format!("address: {msg_address:?}, anycast: {anycast:?}");
        raise_address_error(&ext_addr_str, err_msg)?
    }

    Ok(TonAddress::new(wc, TonHash::from_vec(addr_mutable)?))
}

fn raise_address_error<T: AsRef<str>>(address: &str, msg: T) -> Result<(), TonlibError> {
    Err(TonAddressParseError(address.to_string(), msg.as_ref().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_cell::TonCell;
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn test_ton_address_to_string() -> anyhow::Result<()> {
        let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
        let addr = TonAddress::new(0, bytes);
        assert_eq!(addr.to_hex(), "0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        assert_eq!(addr.to_b64(true, true, true), "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR");
        assert_eq!(addr.to_b64(true, true, false), "EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR");
        assert_eq!(addr.to_b64(true, true, true), addr.to_string());
        Ok(())
    }

    #[test]
    fn test_ton_address_from_str() -> anyhow::Result<()> {
        let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
        let addr = TonAddress::new(0, bytes);
        assert_eq!(TonAddress::from_str("0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?, addr);
        assert_eq!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?, addr);
        assert_eq!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR")?, addr);
        Ok(())
    }

    #[test]
    fn test_ton_address_derive_stonfi_pool() -> anyhow::Result<()> {
        let code_cell = TonCell::from_boc_hex(
            "b5ee9c7201010101002300084202a9338ecd624ca15d37e4a8d9bf677ddc9b84f0e98f05f2fb84c7afe332a281b4",
        )?
        .into_ref();
        let data_cell = TonCell::from_boc_hex("b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79")?.into_ref();
        let address = TonAddress::derive(0, code_cell, data_cell)?;
        let exp_addr = TonAddress::from_str("EQAdltEfzXG_xteLFaKFGd-HPVKrEJqv_FdC7z2roOddRNdM")?;
        assert_eq!(address, exp_addr);
        Ok(())
    }

    #[test]
    fn test_ton_address_crc_error() -> anyhow::Result<()> {
        assert_err!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjra"));
        Ok(())
    }

    #[test]
    fn test_ton_address_serde() -> anyhow::Result<()> {
        let addr_str = "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR";
        let expected_serialized = format!("\"{addr_str}\"");
        let address = TonAddress::from_str(addr_str)?;
        let serial = serde_json::to_string(&address)?;
        assert_eq!(serial, expected_serialized);

        let deserialized: TonAddress = serde_json::from_str(serial.as_str())?;
        assert_eq!(address, deserialized);
        Ok(())
    }

    #[test]
    fn test_ton_address_ord() -> anyhow::Result<()> {
        let address0 = TonAddress::from_str("EQBKwtMZSZurMxGp7FLZ_lM9t54_ECEsS46NLR3qfIwwTnKW")?;
        let address1 = TonAddress::from_str("EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c")?;
        assert_eq!(address0.partial_cmp(&address1), Some(Ordering::Less));
        Ok(())
    }

    #[test]
    fn test_ton_address_to_msg_addr_int() -> anyhow::Result<()> {
        let address = TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?;
        let msg_addr = address.to_msg_address_int();
        let expected = MsgAddressIntStd {
            anycast: None,
            workchain: 0,
            address: TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?,
        }
        .into();
        assert_eq!(msg_addr, expected);
        Ok(())
    }

    #[test]
    fn test_ton_address_to_msg_addr_none() -> anyhow::Result<()> {
        let address = TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?;
        assert_err!(address.to_msg_address_none());
        let address = TonAddress::ZERO;
        assert_ok!(address.to_msg_address_none());
        Ok(())
    }

    #[test]
    fn test_ton_address_tlb_type() -> anyhow::Result<()> {
        for addr in [
            "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR",
            "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c",
        ] {
            let address = TonAddress::from_str(addr)?;
            let cell = address.to_cell()?;
            let parsed_address = TonAddress::from_cell(&cell)?;
            assert_eq!(address, parsed_address);
        }
        Ok(())
    }
}
