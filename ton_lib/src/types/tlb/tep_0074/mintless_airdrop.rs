use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterAddress;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterTLB;
use crate::types::tlb::adapters::Dict;
use crate::types::tlb::block_tlb::msg_address::MsgAddressInt;
use crate::types::tlb::block_tlb::var_len::VarLenBytes;
use num_bigint::BigUint;
use std::collections::HashMap;
use ton_lib_macros::TLBDerive;

#[derive(Clone, Debug, TLBDerive)]
pub struct MintlessAirdropDict {
    #[tlb_derive(adapter = "Dict::<DictKeyAdapterAddress, DictValAdapterTLB, _, _>::new(256)")]
    pub data: HashMap<MsgAddressInt, MintlessAirdropData>,
}

#[derive(Clone, Debug, PartialEq, TLBDerive)]
pub struct MintlessAirdropData {
    pub amount: VarLenBytes<BigUint, 5>,
    pub start_from: u64,
    pub expired_at: u64,
}
