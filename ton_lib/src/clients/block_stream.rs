use crate::clients::lite_client::client::LiteClient;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::block::block_id_ext::BlockIdExt;
use crate::types::tlb::block_tlb::block::Block;
use crate::types::tlb::TLB;
use std::collections::HashSet;

pub struct BlockStream {
    client: LiteClient,
    to_mc_seqno: u32,
    prev_mc_seqno: u32,
    prev_shards: HashSet<BlockIdExt>,
}

impl BlockStream {
    pub async fn new(client: LiteClient, from_seqno: u32, to_seqno: Option<u32>) -> Result<Self, TonlibError> {
        let mut stream = Self {
            client,
            to_mc_seqno: to_seqno.unwrap_or(u32::MAX),
            prev_mc_seqno: from_seqno - 1,
            prev_shards: HashSet::new(),
        };
        if from_seqno > 1 {
            let prev_shards = stream.get_shard_ids(from_seqno - 1).await?;
            stream.prev_shards = prev_shards.into_iter().collect();
        }
        Ok(stream)
    }

    // mc_block_id always first
    pub async fn next(&mut self) -> Result<Option<Vec<BlockIdExt>>, TonlibError> {
        let next_mc_seqno = self.prev_mc_seqno + 1;
        if next_mc_seqno > self.to_mc_seqno {
            return Ok(None); // stream is over
        }
        let shard_ids = self.get_shard_ids(next_mc_seqno).await?;
        self.prev_mc_seqno = next_mc_seqno;
        self.prev_shards = shard_ids.iter().cloned().collect();
        Ok(Some(shard_ids))
    }

    pub async fn get_shard_ids(&mut self, mc_seqno: u32) -> Result<Vec<BlockIdExt>, TonlibError> {
        let mc_block_id = self.client.lookup_mc_block(mc_seqno).await?;
        let block_data = self.client.get_block(mc_block_id, None).await?;
        extract_unseen_ids(&Block::from_boc(&block_data.data)?, &self.prev_shards)
    }
}

fn extract_unseen_ids(mc_block: &Block, prev_shards: &HashSet<BlockIdExt>) -> Result<Vec<BlockIdExt>, TonlibError> {
    let block_extra = mc_block.extra.mc_block_extra.as_ref().unwrap(); // safe on mc_block
    todo!();
}
