use crate::tlb_adapters::ConstLen;
use num_bigint::BigUint;
use ton_lib_core::types::TonAddress;
use ton_lib_core::TLBDerive;

/// ```raw
/// report_static_data#0x8b771735
///   query_id:uint64
///   index:uint256
///   collection:MsgAddress
/// = InternalMsgBody
/// ```
#[derive(Clone, Debug, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0x8b771735, bits_len = 32, ensure_empty = true)]
pub struct NftReportStaticDataMsg {
    pub query_id: u64,
    #[tlb_derive(bits_len = 256)]
    pub index: BigUint, // numerical index of this NFT in the collection, usually serial number of deployment.
    pub collection: TonAddress, // address of the smart contract of the collection to which this NFT belongs.
}

impl NftReportStaticDataMsg {
    pub fn new(index: BigUint, collection: TonAddress) -> Self {
        NftReportStaticDataMsg {
            query_id: 0,
            index,
            collection,
        }
    }
}
