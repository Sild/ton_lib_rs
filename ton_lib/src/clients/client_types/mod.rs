use crate::cell::ton_hash::TonHash;
use crate::types::tlb::block_tlb::block::block_id_ext::BlockIdExt;
use crate::types::ton_address::TonAddress;

mod liteapi_serde;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId {
    pub wc: i32,
    pub shard: u64,
    pub seqno: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZeroStateIdExt {
    pub workchain: i32,
    pub root_hash: TonHash,
    pub file_hash: TonHash,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MasterchainInfo {
    pub last: BlockIdExt,
    pub state_root_hash: TonHash,
    pub init: ZeroStateIdExt,
}

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
pub enum TxId {
    LTHash(TxIdLTHash),
    LTAddress { lt: i64, address: TonAddress },
    ExtInMsgHash { hash: TonHash },
    ExtInMsgHashNorm { hash: TonHash },
}
