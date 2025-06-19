use crate::cell::TonHash;
use crate::types::TonAddress;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TxIdLTHash {
    pub lt: i64,
    pub hash: TonHash,
}

impl TxIdLTHash {
    pub const ZERO: Self = Self::new(0, TonHash::ZERO);
    pub const fn new(lt: i64, hash: TonHash) -> Self { Self { lt, hash } }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TxIdLTAddress {
    pub lt: i64,
    pub address: TonAddress,
}

impl TxIdLTAddress {
    pub const fn new(lt: i64, address: TonAddress) -> Self { Self { lt, address } }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TxId {
    LTHash(TxIdLTHash),
    LTAddress(TxIdLTAddress),
    ExtInMsgHash { hash: TonHash },
    ExtInMsgHashNorm { hash: TonHash },
}
