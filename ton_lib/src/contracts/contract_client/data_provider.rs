use crate::cell::ton_hash::TonHash;
use crate::clients::client_types::TxId;
use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::TLClient;
use crate::contracts::contract_client::block_stream::BlockStream;
use crate::contracts::contract_client::types::ContractState;
use crate::emulators::tvm::tvm_response::TVMRunGetMethodSuccess;
use crate::errors::TonlibError;
use crate::types::tlb::TLB;
use crate::types::ton_address::TonAddress;
use async_trait::async_trait;
use std::collections::HashMap;

/// If Optional argument is not specified, returns data for the latest mc_seqno
#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn get_latest_mc_seqno(&self) -> Result<u32, TonlibError>;
    /// returns latest state if tx_id is None
    async fn get_state(&self, address: &TonAddress, tx_id: Option<&TxId>) -> Result<ContractState, TonlibError>;
    /// return latest config if mc_seqno is not specified
    async fn get_config_boc(&self, mc_seqno: Option<u32>) -> Result<Vec<u8>, TonlibError>;
    /// Is not supposed to check if all required libs were received
    async fn get_libs_boc(&self, lib_ids: &[TonHash], mc_seqno: Option<u32>) -> Result<Option<Vec<u8>>, TonlibError>;
    /// returns latest tx_id for each affected contract at specified mc_seqno
    async fn get_latest_txs(&self, mc_seqno: u32) -> Result<HashMap<TonAddress, TxId>, TonlibError>;

    async fn run_get_method(
        &self,
        address: &TonAddress,
        method: &str,
        stack_boc: Vec<u8>,
    ) -> Result<TVMRunGetMethodSuccess, TonlibError>;
}

pub struct TLDataProvider {
    tl_client: TLClient,
    _block_stream: BlockStream,
}

impl TLDataProvider {
    pub fn new(tl_client: TLClient, block_stream: BlockStream) -> Self {
        Self {
            tl_client,
            _block_stream: block_stream,
        }
    }
}

#[async_trait]
impl DataProvider for TLDataProvider {
    async fn get_latest_mc_seqno(&self) -> Result<u32, TonlibError> {
        Ok(self.tl_client.get_mc_info().await?.last.seqno)
    }

    async fn get_state(&self, address: &TonAddress, tx_id: Option<&TxId>) -> Result<ContractState, TonlibError> {
        let state_raw = match tx_id {
            Some(id) => self.tl_client.get_account_state_raw_by_tx(address.clone(), id.clone().try_into()?).await?,
            None => self.tl_client.get_account_state_raw(address.clone()).await?,
        };
        Ok(ContractState {
            address: address.clone(),
            mc_seqno: state_raw.block_id.seqno,
            last_tx_id: state_raw.last_tx_id.into(),
            code_boc: if state_raw.code.is_empty() {
                None
            } else {
                Some(state_raw.code)
            },
            data_boc: if state_raw.data.is_empty() {
                None
            } else {
                Some(state_raw.data)
            },
            frozen_hash: if state_raw.frozen_hash.is_empty() {
                None
            } else {
                Some(TonHash::from_vec(state_raw.frozen_hash)?)
            },
            balance: state_raw.balance,
        })
    }

    async fn get_config_boc(&self, _mc_seqno: Option<u32>) -> Result<Vec<u8>, TonlibError> {
        // tonlib doesn't support config for particular mc_seqno
        self.tl_client.get_config_boc_all(0).await
    }

    async fn get_libs_boc(&self, lib_ids: &[TonHash], _mc_seqno: Option<u32>) -> Result<Option<Vec<u8>>, TonlibError> {
        // doesn't support libs for particular mc_seqno
        let libs_dict = self.tl_client.get_libs(lib_ids.to_vec()).await?;
        libs_dict.map(|x| x.to_boc()).transpose()
    }

    async fn get_latest_txs(&self, _mc_seqno: u32) -> Result<HashMap<TonAddress, TxId>, TonlibError> {
        // let block_txs = self.0.get_b
        todo!()
    }

    async fn run_get_method(
        &self,
        _address: &TonAddress,
        _method: &str,
        _stack_boc: Vec<u8>,
    ) -> Result<TVMRunGetMethodSuccess, TonlibError> {
        todo!()
    }
}
