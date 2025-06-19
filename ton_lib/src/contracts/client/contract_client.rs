use crate::block_tlb::TVMStack;
use crate::contracts::client::contract_client_cache::*;
use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::tvm::tvm_method_id::TVMGetMethodID;
use crate::emulators::tvm::tvm_response::{TVMRunGetMethodResponse, TVMRunGetMethodSuccess};
use crate::error::TLError;
use std::sync::Arc;
use ton_lib_core::cell::TonHash;
use ton_lib_core::traits::data_provider::{ContractState, DataProvider};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxId};

pub struct ContractClient {
    inner: Arc<Inner>,
}

impl ContractClient {
    pub async fn new(
        cache_config: ContractClientCacheConfig,
        data_provider: Arc<dyn DataProvider>,
    ) -> Result<Self, TLError> {
        let latest_mc_seqno = data_provider.get_latest_mc_seqno().await?;
        Ok(Self {
            inner: Arc::new(Inner {
                data_provider: data_provider.clone(),
                cache: ContractClientCache::new(cache_config, data_provider, latest_mc_seqno)?,
            }),
        })
    }

    pub async fn get_contract_state(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxId>,
    ) -> Result<Arc<ContractState>, TLError> {
        self.inner.cache.get_state(address, tx_id).await
    }

    pub async fn get_config_boc(&self, mc_seqno: Option<u32>) -> Result<EmulBCConfig, TLError> {
        EmulBCConfig::from_boc(&self.inner.data_provider.get_config_boc(mc_seqno).await?)
    }

    pub async fn get_libs_boc(&self, lib_ids: &[TonHash]) -> Result<Option<Vec<u8>>, TLError> {
        Ok(self.inner.data_provider.get_libs_boc(lib_ids, None).await?)
    }

    pub async fn run_get_method<M>(
        &self,
        address: &TonAddress,
        method: M,
        stack: &TVMStack,
    ) -> Result<TVMRunGetMethodSuccess, TLError>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        let method_id = method.into().to_id();
        let rsp = self.inner.data_provider.run_get_method(address, method_id, stack.to_boc()?).await?;
        TVMRunGetMethodResponse::from_json(rsp)?.into_success()
    }

    pub fn get_cache_stats(&self) -> ContractClientCacheStats { self.inner.cache.get_stats() }
}

struct Inner {
    data_provider: Arc<dyn DataProvider>,
    cache: ContractClientCache,
}
