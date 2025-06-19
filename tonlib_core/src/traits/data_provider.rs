use crate::cell::TonHash;
use crate::error::TonlibError;
use crate::types::{TonAddress, TxId};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn get_latest_mc_seqno(&self) -> Result<u32, TonlibError>;
    /// returns latest state if tx_id is None
    async fn get_state(&self, address: &TonAddress, tx_id: Option<&TxId>) -> Result<ContractState, TonlibError>;
    /// return latest config_types if mc_seqno is not specified
    async fn get_config_boc(&self, mc_seqno: Option<u32>) -> Result<Vec<u8>, TonlibError>;
    /// Is not supposed to check if all required libs were received
    async fn get_libs_boc(&self, lib_ids: &[TonHash], mc_seqno: Option<u32>) -> Result<Option<Vec<u8>>, TonlibError>;
    /// returns latest tx_id for each affected contract at specified mc_seqno
    async fn get_latest_txs(&self, mc_seqno: u32) -> Result<HashMap<TonAddress, TxId>, TonlibError>;

    /// returns raw emulator response (json)
    async fn run_get_method(
        &self,
        address: &TonAddress,
        method: i32,
        stack_boc: Vec<u8>,
    ) -> Result<String, TonlibError>;
}

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
