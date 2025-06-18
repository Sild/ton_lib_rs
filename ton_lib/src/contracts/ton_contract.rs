use crate::cell::ton_cell::TonCell;
use crate::cell::ton_cell_utils::TonCellUtils;
use crate::clients::client_types::TxId;
use crate::contracts::contract_client::types::ContractState;
use crate::contracts::contract_client::ContractClient;
use crate::emulators::tvm::tvm_c7::TVMEmulatorC7;
use crate::emulators::tvm::tvm_emulator::TVMEmulator;
use crate::emulators::tvm::tvm_method_id::TVMGetMethodID;
use crate::emulators::tvm::tvm_response::TVMRunGetMethodSuccess;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::tvm::tvm_stack::TVMStack;
use crate::types::tlb::TLB;
use crate::types::ton_address::TonAddress;
use std::sync::Arc;

pub struct ContractCtx {
    pub client: ContractClient,
    pub address: TonAddress,
    pub tx_id: Option<TxId>,
}

#[async_trait::async_trait]
pub trait TonContractTrait: Send + Sync + Sized {
    fn ctx(&self) -> &ContractCtx;
    fn ctx_mut(&mut self) -> &mut ContractCtx;
    fn from_ctx(ctx: ContractCtx) -> Self;

    async fn new(client: ContractClient, address: TonAddress, tx_id: Option<TxId>) -> Result<Self, TonlibError> {
        Ok(Self::from_ctx(ContractCtx { client, address, tx_id }))
    }

    async fn run_get_method<M>(&self, method: M, stack: &TVMStack) -> Result<TVMRunGetMethodSuccess, TonlibError>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        let ctx = self.ctx();
        ctx.client.run_get_method(&ctx.address, method, stack).await
    }

    async fn get_state(&self) -> Result<Arc<ContractState>, TonlibError> {
        let ctx = self.ctx();
        ctx.client.get_contract_state(&ctx.address, ctx.tx_id.as_ref()).await
    }

    async fn get_parsed_data<D: TLB>(&self) -> Result<D, TonlibError> {
        match &self.get_state().await?.data_boc {
            Some(data_boc) => D::from_boc(data_boc),
            None => Err(TonlibError::TonContractNotActive {
                address: self.ctx().address.clone(),
                tx_id: self.ctx().tx_id.clone(),
            }),
        }
    }

    #[cfg(feature = "emulator")]
    async fn make_emulator(&self, c7: Option<&TVMEmulatorC7>) -> Result<TVMEmulator, TonlibError> {
        let ctx = self.ctx();
        let state = self.get_state().await?;
        let code_boc = state.code_boc.as_deref().unwrap_or(&[]);
        let code_cell = state.code_boc.as_ref().map(|x| TonCell::from_boc(x)).transpose()?;

        let data_boc = state.data_boc.as_deref().unwrap_or(&[]);
        let data_cell = state.data_boc.as_ref().map(|x| TonCell::from_boc(x)).transpose()?;

        let mut emulator = match c7 {
            Some(c7) => TVMEmulator::new(code_boc, data_boc, c7)?,
            None => {
                let bc_config = ctx.client.get_config_boc(None).await?;
                let c7 = TVMEmulatorC7::new(ctx.address.clone(), bc_config)?;
                TVMEmulator::new(code_boc, data_boc, &c7)?
            }
        };
        let cells = [code_cell.as_ref(), data_cell.as_ref()].into_iter().flatten();
        let lib_ids = TonCellUtils::extract_lib_ids(cells)?;
        if !lib_ids.is_empty() {
            if let Some(libs_boc) = ctx.client.get_libs_boc(&lib_ids.into_iter().collect::<Vec<_>>()).await? {
                emulator.set_libs(&libs_boc)?;
            }
        }
        Ok(emulator)
    }
}
