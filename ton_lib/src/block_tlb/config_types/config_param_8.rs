use ton_lib_core::TLBDerive;

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0xc4, bits_len = 8)]
pub struct GlobalVersion {
    pub version: u32,
    pub capabilities: u64,
}
