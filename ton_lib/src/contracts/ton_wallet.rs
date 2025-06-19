use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::ContractCtx;
use crate::contracts::ton_contract::TonContractTrait;
use crate::error::TLError;
use ton_lib_core::cell::TonHash;
use ton_lib_core::ton_contract;

#[ton_contract]
pub struct WalletContract {}

impl WalletContract {
    pub async fn seqno(&self) -> Result<u32, TLError> {
        let result = self.run_get_method("seqno", &TVMStack::default()).await?;
        let seqno_int = result.stack_parsed()?.pop_tiny_int()?;
        if seqno_int < 0 {
            return Err(TLError::UnexpectedValue {
                expected: "non-negative integer".to_string(),
                actual: seqno_int.to_string(),
            });
        }
        Ok(seqno_int as u32)
    }

    pub async fn get_public_key(&self) -> Result<TonHash, TLError> {
        let result = self.run_get_method("get_public_key", &TVMStack::default()).await?;
        Ok(TonHash::from_num(&result.stack_parsed()?.pop_int()?)?)
    }
}
