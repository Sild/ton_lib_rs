use crate::block_tlb::TVMStack;
use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::TLClient;
use crate::contracts::tl_provider::provider_cache::{EmulateGetMethodArgs, StateCache};
use crate::contracts::tl_provider::provider_config::TLProviderConfig;
use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::tvm::tvm_c7::TVMEmulatorC7;
use crate::emulators::tvm::tvm_emulator::TVMEmulator;
use crate::emulators::tvm::tvm_response::TVMGetMethodSuccess;
use crate::error::TLError;
use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use ton_lib_core::cell::{TonCell, TonCellUtils, TonHash};
use ton_lib_core::error::TLCoreError;
use ton_lib_core::traits::contract_provider::{
    ContractMethodArgs, ContractMethodResponse, ContractMethodState, ContractProvider, ContractState,
};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxIdLTHash};

pub struct TLProvider {
    tl_client: TLClient,
    cache: Arc<StateCache>,
    bc_config: EmulBCConfig,
}

#[async_trait]
impl ContractProvider for TLProvider {
    async fn get_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxIdLTHash>,
    ) -> Result<Arc<ContractState>, TLCoreError> {
        let state = match tx_id {
            Some(id) => self.cache.get_by_tx(address, id).await,
            None => self.cache.get_latest(address).await,
        }?;
        Ok(state)
    }

    async fn run_get_method(&self, args: ContractMethodArgs) -> Result<ContractMethodResponse, TLCoreError> {
        let state = match args.method_state {
            ContractMethodState::Latest => self.get_contract(&args.address, None).await?,
            ContractMethodState::TxId(id) => self.get_contract(&args.address, Some(&id)).await?,
            ContractMethodState::Custom(state) => state,
        };

        let emulate_args = EmulateGetMethodArgs::new(state, args.method_id, args.stack_boc.map(Arc::new));
        self.cache.cache_stats.emulate_get_method_req.fetch_add(1, Relaxed);
        let success = self
            .cache
            .emulate_get_method_cache
            .try_get_with_by_ref(&emulate_args, async {
                self.cache.cache_stats.emulate_get_method_miss.fetch_add(1, Relaxed);
                self.emulate_get_method(
                    &emulate_args.state,
                    emulate_args.method_id,
                    emulate_args.stack.as_ref().map(|s| s.as_slice()).unwrap_or(TVMStack::EMPTY_BOC),
                )
                .await
            })
            .await
            .map_err(|e| TLCoreError::from(e.deref()))?;

        Ok(ContractMethodResponse {
            exit_code: success.vm_exit_code,
            stack_boc: BASE64_STANDARD.decode(success.stack_boc_base64)?,
        })
    }

    async fn get_cache_stats(&self) -> Result<HashMap<String, usize>, TLCoreError> {
        let latest_entry_count = self.cache.state_latest_cache.entry_count() as usize;
        let by_tx_entry_count = self.cache.state_by_tx_cache.entry_count() as usize;
        let emulate_get_method_entry_count = self.cache.emulate_get_method_cache.entry_count() as usize;
        Ok(self.cache.cache_stats.export(latest_entry_count, by_tx_entry_count, emulate_get_method_entry_count))
    }
}

impl TLProvider {
    pub async fn new(config: TLProviderConfig, tl_client: TLClient) -> Result<Self, TLError> {
        let bc_config = tl_client.get_config_boc_all(0).await?;
        log::info!("[tl_provider]: bc_config received ({} bytes)", bc_config.len());
        let cache = StateCache::new(config, tl_client.clone()).await?;
        Ok(Self {
            tl_client,
            cache,
            bc_config: EmulBCConfig::from_boc(&bc_config)?,
        })
    }

    async fn emulate_get_method(
        &self,
        state: &ContractState,
        method: i32,
        stack: &[u8],
    ) -> Result<TVMGetMethodSuccess, TLError> {
        let code_boc = match &state.code_boc {
            Some(boc) => boc,
            None => return Err(TLCoreError::ContractError("code is None at state: {state:?}".to_string()).into()),
        };

        let data_boc = state.data_boc.as_deref().unwrap_or(&[]);

        let c7 = TVMEmulatorC7 {
            address: state.address.clone(),
            unix_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(TLCoreError::from)?
                .as_secs() as u32,
            balance: state.balance as u64,
            rand_seed: TonHash::ZERO,
            config: self.bc_config.clone(),
        };

        let mut emulator = TVMEmulator::new(code_boc, data_boc, &c7)?;
        if let Some(libs) = self.get_libs_boc(code_boc, data_boc).await? {
            emulator.set_libs(&libs)?;
        }
        emulator.run_get_method(method, stack)
    }

    async fn get_libs_boc(&self, code_boc: &[u8], data_boc: &[u8]) -> Result<Option<Vec<u8>>, TLCoreError> {
        let code = TonCell::from_boc(code_boc)?;
        let data = if data_boc.is_empty() {
            None
        } else {
            Some(TonCell::from_boc(data_boc)?)
        };
        let cells = [Some(&code), data.as_ref()].into_iter().flatten();
        let lib_ids = TonCellUtils::extract_lib_ids(cells)?;
        let libs = if lib_ids.is_empty() {
            return Ok(None);
        } else {
            self.tl_client.get_libs(lib_ids.into_iter().collect()).await?
        };
        libs.map(|x| x.to_boc()).transpose()
    }
}
