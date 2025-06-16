use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use crate::types::tlb::adapters::bin_tree::BinTree;
use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterInto;
use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterTLB;
use crate::types::tlb::adapters::tlb_hash_map_e::TLBHashMapE;
use crate::types::tlb::block_tlb::block::block_id_ext::BlockIdExt;
use crate::types::tlb::block_tlb::block::shard_descr::ShardDescr;
use crate::types::tlb::block_tlb::block::shard_ident::{ShardIdent, ShardPfx};
use crate::types::tlb::block_tlb::coins::CurrencyCollection;
use crate::types::tlb::block_tlb::config::config_params::ConfigParams;
use crate::types::tlb::{TLBPrefix, TLB};
use std::collections::HashMap;
use ton_lib_macros::TLBDerive;

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L593
#[derive(Debug, Clone, PartialEq)]
pub struct MCBlockExtra {
    pub key_block: bool,
    pub shard_hashes: HashMap<i32, HashMap<ShardPfx, ShardDescr>>, // wc_id -> shard_pfx -> ShardDescr
    pub shard_fees: Option<TonCellRef>,
    shard_fees_crated: ShardFeesCreated, // this is a mock to read/write cell properly while we don't support a fair HashmapAugE
    // https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L597
    pub ref_data: TonCellRef,
    pub config: Option<ConfigParams>,
}

impl MCBlockExtra {
    pub fn shard_ids(&self) -> Vec<BlockIdExt> {
        let mut shard_ids = vec![];
        for (wc, shards) in &self.shard_hashes {
            for (shard_pfx, descr) in shards {
                shard_ids.push(BlockIdExt {
                    shard_id: ShardIdent::from_pfx(*wc, shard_pfx),
                    seqno: descr.seqno,
                    root_hash: TonHash::from_slice_sized(descr.root_hash.as_slice_sized()),
                    file_hash: TonHash::from_slice_sized(descr.file_hash.as_slice_sized()),
                });
            }
        }
        shard_ids
    }
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub struct ShardFeesCreated {
    pub fees: CurrencyCollection,
    pub create: CurrencyCollection,
}

impl TLB for MCBlockExtra {
    const PREFIX: TLBPrefix = TLBPrefix::new(0xcca5, 16);
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        let key_block = TLB::read(parser)?;
        let shards_dict = TLBHashMapE::<DictKeyAdapterInto, DictValAdapterTLB, u32, TonCellRef>::new(32);
        let mut shard_hashes = HashMap::new();
        for (wc_id, cell_ref) in shards_dict.read(parser)? {
            let cur_hashes = BinTree::<DictValAdapterTLB, ShardDescr>::new().read(&mut cell_ref.parser())?;
            shard_hashes.insert(wc_id as i32, cur_hashes);
        }
        let shard_fees = TLB::read(parser)?;
        let shard_fees_crated = TLB::read(parser)?;
        let ref_data = TLB::read(parser)?;

        let config = match key_block {
            true => Some(TLB::read(parser)?),
            false => None,
        };
        Ok(Self {
            key_block,
            shard_hashes,
            shard_fees,
            shard_fees_crated,
            ref_data,
            config,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> {
        self.key_block.write(builder)?;
        let mut shards_dict = HashMap::<u32, TonCellRef>::new();
        for (wc_id, shards) in &self.shard_hashes {
            let mut val_builder = TonCell::builder();
            BinTree::<DictValAdapterTLB, _>::new().write(&mut val_builder, shards)?;
            shards_dict.insert(*wc_id as u32, val_builder.build_ref()?);
        }
        TLBHashMapE::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32).write(builder, &shards_dict)?;
        self.shard_fees.write(builder)?;
        self.shard_fees_crated.write(builder)?;
        self.ref_data.write(builder)?;
        if self.key_block {
            match &self.config {
                Some(config) => config.write(builder)?,
                None => {
                    let err_msg = "MCBlockExtra has key_block=true, but config is None".to_string();
                    return Err(TonlibError::TLBWrongData(err_msg));
                }
            }
        }
        Ok(())
    }
}
