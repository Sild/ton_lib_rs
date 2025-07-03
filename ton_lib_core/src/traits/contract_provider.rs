use crate::cell::TonHash;
use crate::error::TLCoreError;
use crate::types::{TonAddress, TxIdLTHash};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait ContractProvider: Send + Sync + 'static {
    /// if tx_id is None, returns latest state
    async fn get_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxIdLTHash>,
    ) -> Result<Arc<ContractState>, TLCoreError>;
    async fn run_get_method(&self, args: ContractMethodArgs) -> Result<ContractMethodResponse, TLCoreError>;
    async fn get_cache_stats(&self) -> Result<HashMap<String, usize>, TLCoreError>;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ContractMethodArgs {
    pub address: TonAddress,
    pub method_state: ContractMethodState,
    pub method_id: i32,
    pub stack_boc: Option<Vec<u8>>,
}

impl ContractMethodArgs {
    /// uses latest state if tx_id is None
    pub fn new(address: TonAddress, tx_id: Option<TxIdLTHash>, method_id: i32, stack_boc: Option<Vec<u8>>) -> Self {
        let method_state = match tx_id {
            Some(tx_id) => ContractMethodState::TxId(tx_id),
            None => ContractMethodState::Latest,
        };
        Self {
            address,
            method_state,
            method_id,
            stack_boc,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ContractMethodState {
    Latest,
    TxId(TxIdLTHash),
    Custom(Arc<ContractState>),
}

#[derive(Debug, Clone)]
pub struct ContractMethodResponse {
    pub exit_code: i32,
    pub stack_boc: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ContractState {
    pub address: TonAddress,
    pub mc_seqno: u32,
    pub last_tx_id: TxIdLTHash,
    pub code_boc: Option<Vec<u8>>,
    pub data_boc: Option<Vec<u8>>,
    pub frozen_hash: Option<TonHash>,
    pub balance: i64,
}
