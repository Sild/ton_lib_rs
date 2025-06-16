use crate::cell::ton_cell::TonCellRef;
use crate::cell::ton_hash::TonHash;
use crate::types::tlb::adapters::TLBRefOpt;
use crate::types::tlb::block_tlb::block::mc_block_extra::MCBlockExtra;
use ton_lib_macros::TLBDerive;

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L467
#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0x4a33f6fd, bits_len = 32)]
pub struct BlockExtra {
    pub in_msg_descr: TonCellRef,   // TODO
    pub out_msg_descr: TonCellRef,  // TODO
    pub account_blocks: TonCellRef, // TODO
    pub rand_seed: TonHash,
    pub created_by: TonHash,
    #[tlb_derive(adapter = "TLBRefOpt")]
    pub mc_block_extra: Option<MCBlockExtra>,
}
