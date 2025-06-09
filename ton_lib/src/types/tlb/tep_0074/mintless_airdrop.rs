use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterAddress;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterTLB;
use crate::types::tlb::adapters::ConstLen;
use crate::types::tlb::adapters::Dict;
use crate::types::tlb::block_tlb::coins::Coins;
use crate::types::tlb::block_tlb::msg_address::MsgAddressInt;
use std::collections::HashMap;
use ton_lib_macros::TLBDerive;

#[derive(Clone, Debug, TLBDerive)]
pub struct MintlessAirdropDict {
    #[tlb_derive(adapter = "Dict::<DictKeyAdapterAddress, DictValAdapterTLB, _, _>::new(267)")]
    pub data: HashMap<MsgAddressInt, MintlessAirdropData>,
}

#[derive(Clone, Debug, PartialEq, TLBDerive)]
pub struct MintlessAirdropData {
    pub amount: Coins,
    #[tlb_derive(bits_len = 48)]
    pub start_from: u64,
    #[tlb_derive(bits_len = 48)]
    pub expired_at: u64,
}
