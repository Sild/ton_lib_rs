use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContractTrait;
use crate::error::TLError;
use async_trait::async_trait;
use std::ops::Deref;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;

#[async_trait]
pub trait GetWalletAddress: TonContractTrait {
    async fn get_wallet_address(&self, owner: &TonAddress) -> Result<TonAddress, TLError> {
        let mut stack = TVMStack::default();
        stack.push_cell_slice(owner.to_cell_ref()?);
        let run_result = self.run_get_method("get_wallet_address", &stack).await?;
        Ok(TonAddress::from_cell(run_result.stack_parsed()?.pop_cell()?.deref())?)
    }
}
