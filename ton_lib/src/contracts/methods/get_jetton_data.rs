use crate::block_tlb::{Coins, TVMStack};
use crate::contracts::ton_contract::TonContractTrait;
use crate::error::TLError;
use async_trait::async_trait;
use std::ops::Deref;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;

#[async_trait]
pub trait GetJettonData: TonContractTrait {
    async fn get_jetton_data(&self) -> Result<GetJettonDataResult, TLError> {
        let run_result = self.run_get_method("get_jetton_data", &TVMStack::default()).await?;
        GetJettonDataResult::from_stack(&mut run_result.stack_parsed()?)
    }
}

pub struct GetJettonDataResult {
    pub total_supply: Coins,
    pub mintable: bool,
    pub admin: TonAddress,
    pub content: TonCellRef,
    pub wallet_code: TonCellRef,
}

impl GetJettonDataResult {
    pub fn from_stack(stack: &mut TVMStack) -> Result<Self, TLError> {
        let wallet_code = stack.pop_cell()?;
        let content = stack.pop_cell()?;
        let admin = TonAddress::from_cell(stack.pop_cell()?.deref())?;
        let mintable = stack.pop_tiny_int()? != 0;
        let total_supply = Coins::from_signed(stack.pop_tiny_int()?)?;

        Ok(Self {
            total_supply,
            mintable,
            admin,
            content,
            wallet_code,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_jetton_data_result() -> anyhow::Result<()> {
        let mut stack = TVMStack::from_boc_hex("b5ee9c720102100100010100020800000503010e02020302030209040f1470200405010300c006011201ffffffffffffffff070253705148e3baabcb0800c881fc78d28207072c728a2e7896228f37e17369ae121cb0eef7b4b0385f3330400e08020120090a0112010005148e3baabcb00b01000f0143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e6400c0143bff7407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f400d00000102000f000400360842028f452d7a4dfd74066b682365177259ed05734435be76b5fd4bd5d8af2b7c3d68003e68747470733a2f2f7465746865722e746f2f757364742d746f6e2e6a736f6e")?;
        let result = GetJettonDataResult::from_stack(&mut stack)?;
        assert_eq!(result.total_supply, Coins::from_str("1429976002510000")?);
        assert!(result.mintable);
        assert_eq!(
            result.admin,
            TonAddress::from_str("0:6440fe3c69410383963945173c4b11479bf0b9b4d7090e58777bda581c2f9998")?
        );
        assert_eq!(result.content, TonCellRef::from_boc_hex("b5ee9c7201010701007d00010300c00102012002030143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e640040143bff7407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f400601020005003e68747470733a2f2f7465746865722e746f2f757364742d746f6e2e6a736f6e00040036")?);
        Ok(())
    }
}
