use crate::cell::ton_hash::TonHash;
use crate::clients::client_types::TxId;
use crate::types::ton_address::TonAddress;

#[derive(Debug, Clone)]
pub struct ContractState {
    pub address: TonAddress,
    pub mc_seqno: u32,
    pub last_tx_id: TxId,
    pub code_boc: Option<Vec<u8>>,
    pub data_boc: Option<Vec<u8>>,
    pub frozen_hash: Option<TonHash>,
    pub balance: i64,
}
