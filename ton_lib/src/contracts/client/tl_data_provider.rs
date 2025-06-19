use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::TLClient;
use crate::contracts::client::block_stream::BlockStream;
use async_trait::async_trait;
use std::collections::HashMap;
use ton_lib_core::cell::TonHash;
use ton_lib_core::error::TonlibError;
use ton_lib_core::traits::data_provider::{ContractState, DataProvider};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxId};

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
            Some(id) => self.tl_client.get_account_state_raw_by_tx(address.clone(), id.clone().try_into()?).await,
            None => self.tl_client.get_account_state_raw(address.clone()).await,
        }?;

        let code_boc = match state_raw.code.is_empty() {
            true => Some(state_raw.code),
            _ => None,
        };

        let data_boc = match state_raw.data.is_empty() {
            true => Some(state_raw.data),
            _ => None,
        };

        let frozen_hash = match state_raw.frozen_hash.is_empty() {
            true => None,
            _ => Some(TonHash::from_vec(state_raw.frozen_hash)?),
        };

        Ok(ContractState {
            address: address.clone(),
            mc_seqno: state_raw.block_id.seqno,
            last_tx_id: state_raw.last_tx_id.into(),
            code_boc,
            data_boc,
            frozen_hash,
            balance: state_raw.balance,
        })
    }

    async fn get_config_boc(&self, _mc_seqno: Option<u32>) -> Result<Vec<u8>, TonlibError> {
        // tonlib doesn't support config_types for particular mc_seqno
        self.tl_client.get_config_boc_all(0).await.map_err(TonlibError::from)
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
        _method: i32,
        _stack_boc: Vec<u8>,
    ) -> Result<String, TonlibError> {
        todo!()
    }
}
