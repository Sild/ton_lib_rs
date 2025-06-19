use crate::block_tlb::BlockExtra;
use crate::block_tlb::BlockInfo;
use crate::tlb_adapters::TLBRef;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::TLBDerive;

// https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L462
#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0x11ef55aa, bits_len = 32)]
pub struct Block {
    pub global_id: i32,
    #[tlb_derive(adapter = "TLBRef")]
    pub info: BlockInfo,
    pub value_flow: TonCellRef,   // TODO
    pub state_update: TonCellRef, // TODO
    #[tlb_derive(adapter = "TLBRef")]
    pub extra: BlockExtra,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::block_tlb::block_types::block_info::ExtBlockRef;
    use crate::block_tlb::GlobalVersion;
    use crate::block_tlb::ShardIdent;
    use crate::block_tlb::_test_block_data::MASTER_BLOCK_BOC_HEX;
    use std::str::FromStr;
    use ton_lib_core::cell::TonHash;
    use ton_lib_core::traits::tlb::TLB;

    #[test]
    fn test_block_tlb_block() -> anyhow::Result<()> {
        let parsed = Block::from_boc_hex(MASTER_BLOCK_BOC_HEX)?;
        assert_eq!(
            parsed.extra.mc_block_extra.as_ref().unwrap().cell_hash()?,
            TonHash::from_str("D0D3EA6B963ABEB1C5E2F9936DB6DA8ADDB1DF1F221F1F13C85120DE0BB79DA0")?
        );
        assert_eq!(
            parsed.cell_hash()?,
            TonHash::from_str("CBEBAA6AC4270C987C90C5ED930FF37F9B73C705999585D6D8C1C5E9FA3DD6E3")?
        );
        // test block.info
        let expected_block_info = BlockInfo {
            version: 0,
            not_master: false,
            after_merge: false,
            before_split: false,
            after_split: false,
            want_split: false,
            want_merge: true,
            key_block: true,
            vert_seqno_incr: false,
            flags: 1,
            seqno: 46991999,
            vert_seqno: 1,
            shard: ShardIdent {
                wc: -1,
                shard: 0x8000000000000000,
            },
            gen_utime: 1745112841,
            start_lt: 56255102000000,
            end_lt: 56255102000004,
            gen_validator_list_has_short: 4143742061,
            gen_catchain_seqno: 682531,
            min_ref_mc_seqno: 46991995,
            prev_key_block_seqno: 46989053,
            gen_software: Some(GlobalVersion {
                version: 10,
                capabilities: 494,
            }),
            master_ref: None,
            prev_ref: ExtBlockRef {
                end_lt: 56255101000004,
                seqno: 46991998,
                root_hash: TonHash::from_str("a16dd643a1b54a6804ce3264503d9feab4e0f5d1de450888f188179557093595")?,
                file_hash: TonHash::from_str("2e58dcf8fe16ccc203ddd1d053984f9ac6eafcf0543ced95f96ab9e7e411d256")?,
            }
            .into(),

            prev_vert_ref: None,
        };
        assert_eq!(expected_block_info, parsed.info);

        assert!(parsed.extra.mc_block_extra.is_some());

        // test block.extra.mc_block_extra.shard_hashes
        let expected_shards = HashMap::from([
            (0x2000000000000000u64, 52077744),
            (0x6000000000000000, 52097945),
            (0xa000000000000000, 51731388),
            (0xe000000000000000, 51757085),
        ]);
        let parsed_shard_hashes = &parsed.extra.mc_block_extra.as_ref().unwrap().shard_hashes;
        assert_eq!(parsed_shard_hashes.len(), 1);
        assert!(parsed_shard_hashes.contains_key(&0));
        let parsed_shard_descr = parsed_shard_hashes.get(&0).unwrap();
        assert_eq!(parsed_shard_descr.len(), expected_shards.len());
        for (shard_pfx, descr) in parsed_shard_descr {
            let shard = shard_pfx.to_shard();
            assert!(expected_shards.contains_key(&shard), "shard: {shard:X}, shard_pfx: {shard_pfx:?}");
            let expected_seqno = expected_shards.get(&shard).unwrap();
            assert_eq!(*expected_seqno, descr.seqno);
        }

        // full serialization test
        let serialized = parsed.to_boc()?;
        let parsed_back = Block::from_boc(&serialized)?;
        assert_eq!(parsed_back, parsed);
        assert_eq!(
            parsed_back.cell_hash()?,
            TonHash::from_str("CBEBAA6AC4270C987C90C5ED930FF37F9B73C705999585D6D8C1C5E9FA3DD6E3")?
        );
        Ok(())
    }
}
