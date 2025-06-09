use crate::cell::ton_hash::TonHash;
use crate::clients::client_types::TxId;
use crate::contracts::contract_client::contract_client_cache::{
    ContractClientCache, ContractClientCacheConfig, ContractClientCacheStats,
};
use crate::contracts::contract_client::data_provider::DataProvider;
use crate::contracts::contract_client::types::ContractState;
use crate::emulators::emul_bc_config::EmulatorBCConfig;
use crate::emulators::tvm::tvm_method_id::TVMGetMethodID;
use crate::emulators::tvm::tvm_response::TVMRunGetMethodSuccess;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::tvm::tvm_stack::TVMStack;
use crate::types::tlb::TLB;
use crate::types::ton_address::TonAddress;
use std::sync::Arc;

pub mod contract_client_cache;
pub mod data_provider;
pub mod types;

pub struct ContractClient {
    inner: Arc<Inner>,
}

impl ContractClient {
    pub async fn new(
        cache_config: ContractClientCacheConfig,
        data_provider: Arc<dyn DataProvider>,
    ) -> Result<Self, TonlibError> {
        let latest_mc_seqno = data_provider.get_latest_mc_seqno().await?;
        Ok(Self {
            inner: Arc::new(Inner {
                data_provider: data_provider.clone(),
                cache: ContractClientCache::new(cache_config, data_provider, latest_mc_seqno)?,
            }),
        })
    }

    pub async fn get_state(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxId>,
    ) -> Result<Arc<ContractState>, TonlibError> {
        self.inner.cache.get_state(address, tx_id).await
    }

    pub async fn get_config_boc(&self, mc_seqno: Option<u32>) -> Result<EmulatorBCConfig, TonlibError> {
        EmulatorBCConfig::from_boc(&self.inner.data_provider.get_config_boc(mc_seqno).await?)
    }

    pub async fn get_libs_boc(&self, lib_ids: &[TonHash]) -> Result<Option<Vec<u8>>, TonlibError> {
        self.inner.data_provider.get_libs_boc(lib_ids, None).await
    }

    pub async fn run_get_method<M>(
        &self,
        address: &TonAddress,
        method: M,
        stack: &TVMStack,
    ) -> Result<TVMRunGetMethodSuccess, TonlibError>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        self.inner.data_provider.run_get_method(address, &method.into().as_str(), stack.to_boc()?).await
    }

    pub fn get_cache_stats(&self) -> ContractClientCacheStats { self.inner.cache.get_stats() }
}

struct Inner {
    data_provider: Arc<dyn DataProvider>,
    cache: ContractClientCache,
}
