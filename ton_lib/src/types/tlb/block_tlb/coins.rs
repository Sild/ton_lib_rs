use crate::types::tlb::adapters::DictRef;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterTLB;
use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterInto;
use crate::types::tlb::adapters::TLBRef;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::var_len::VarLenBytes;
use num_bigint::{BigInt, BigUint};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use ton_lib_macros::TLBDerive;

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L116
#[derive(Clone, Debug, PartialEq, TLBDerive)]
pub struct Grams(pub VarLenBytes<BigUint, 4>);

// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L118
// ¯\_(ツ)_/¯
pub type Coins = Grams;

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L124
#[derive(Clone, Debug, PartialEq, TLBDerive)]
pub struct CurrencyCollection {
    pub grams: Grams,
    #[tlb_derive(adapter = "DictRef::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32)")]
    pub other: HashMap<u32, VarLenBytes<BigUint, 32>>,
}

impl Grams {
    pub fn new<T: Into<BigUint>>(amount: T) -> Self {
        let amount = amount.into();
        let bits_len = amount.bits();
        Self(VarLenBytes::new(amount, bits_len as usize))
    }
    pub fn zero() -> Self { Grams::new(0u32) }
    pub fn from_signed<T: Into<BigInt>>(amount: T) -> Result<Self, TonlibError> {
        let amount = amount.into();
        let unsigned = match amount.to_biguint() {
            Some(amount) => amount,
            None => {
                return Err(TonlibError::UnexpectedValue {
                    expected: "positive int".to_string(),
                    actual: format!("{amount}"),
                })
            }
        };
        Ok(Self::new(unsigned))
    }
}

impl CurrencyCollection {
    pub fn new<T: Into<BigUint>>(grams: T) -> Self {
        Self {
            grams: Grams::new(grams),
            other: Default::default(),
        }
    }
}

impl FromStr for CurrencyCollection {
    type Err = TonlibError;
    fn from_str(grams: &str) -> Result<Self, Self::Err> { Ok(Self::new(BigUint::from_str(grams)?)) }
}

impl Deref for Grams {
    type Target = BigUint;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Grams {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T: Into<BigUint>> From<T> for Grams {
    fn from(value: T) -> Self { Grams::new(value) }
}

impl FromStr for Grams {
    type Err = TonlibError;
    fn from_str(grams: &str) -> Result<Self, Self::Err> { Ok(Self::new(BigUint::from_str(grams)?)) }
}

#[cfg(test)]
mod tests {
    use crate::types::tlb::block_tlb::coins::CurrencyCollection;
    use crate::types::tlb::tlb_type::TLBType;

    #[test]
    fn test_currency_collection() -> anyhow::Result<()> {
        let parsed = CurrencyCollection::from_boc_hex("b5ee9c720101010100070000094c143b1d14")?;
        assert_eq!(*parsed.grams, 3242439121u32.into());

        let cell_serial = parsed.to_cell()?;
        let parsed_back = CurrencyCollection::from_cell(&cell_serial)?;
        assert_eq!(parsed, parsed_back);
        Ok(())
    }

    #[test]
    fn test_currency_collection_zero_grams() -> anyhow::Result<()> {
        let currency = CurrencyCollection::new(0u32);
        let cell = currency.to_cell()?;
        let parsed = CurrencyCollection::from_cell(&cell)?;
        assert_eq!(*parsed.grams, 0u32.into());

        let cell_serial = parsed.to_cell()?;
        assert_eq!(cell_serial, cell);
        Ok(())
    }
}
