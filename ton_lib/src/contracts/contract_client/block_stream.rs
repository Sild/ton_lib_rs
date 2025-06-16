use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::block::block_id_ext::BlockIdExt;
use std::collections::HashSet;
use std::time::Duration;
use async_recursion::async_recursion;
use futures_util::future::try_join_all;
use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::{TLClient, TLConnection};

pub struct BlockStream {
    client: TLClient,
    to_mc_seqno: u32,
    prev_mc_seqno: u32,
    prev_shards: HashSet<BlockIdExt>,
}

impl BlockStream {
    pub async fn new(client: TLClient, from_seqno: u32, to_seqno: Option<u32>) -> Result<Self, TonlibError> {
        let mut stream = Self {
            client,
            to_mc_seqno: to_seqno.unwrap_or(u32::MAX),
            prev_mc_seqno: from_seqno - 1,
            prev_shards: HashSet::new(),
        };
        if from_seqno > 1 {
            let (conn, mc_block_id) = stream.find_connection(from_seqno - 1).await?;
            let shard_ids = conn.get_block_shards(mc_block_id).await?.shards;
            stream.prev_shards = shard_ids.into_iter().collect();
        }
        Ok(stream)
    }

    // mc_block_id always last
    pub async fn next(&mut self) -> Result<Option<Vec<BlockIdExt>>, TonlibError> {
        let next_mc_seqno = self.prev_mc_seqno + 1;
        if next_mc_seqno > self.to_mc_seqno {
            return Ok(None); // stream is over
        }

        let (conn, mc_block_id) = self.find_connection(next_mc_seqno).await?;
        let shard_ids = conn.get_block_shards(mc_block_id.clone()).await?.shards;
        let mut unseen_shards = self.get_unseen_shards(conn, &shard_ids).await?;
        unseen_shards.extend(shard_ids);

        self.prev_mc_seqno = next_mc_seqno;
        self.prev_shards = unseen_shards.clone();

        let mut result: Vec<BlockIdExt> = unseen_shards.into_iter().collect();
        result.push(mc_block_id);
        Ok(Some(result))
    }

    async fn find_connection(&self, mc_seqno: u32) -> Result<(&TLConnection, BlockIdExt), TonlibError> {
        loop {
            let conn = self.client.get_connection();
            let mc_info = conn.get_mc_info().await?;
            if mc_info.last.seqno >= mc_seqno {
                return Ok((conn, mc_info.last));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        };
    }

    #[async_recursion]
    async fn get_unseen_shards(&self, conn: &TLConnection, shard_ids: &[BlockIdExt]) -> Result<HashSet<BlockIdExt>, TonlibError> {
        let get_prev_ids_futs = shard_ids.iter().map(|block_id| async {
            
            if self.prev_shards.contains(block_id) {
                return Ok::<_, TonlibError>(Default::default());
            }
            
            let prev_ids = conn.get_block_header(block_id.clone()).await?.prev_blocks.unwrap_or_default();
            let mut unseen_prev_ids = self.get_unseen_shards(conn, &prev_ids).await?;
            unseen_prev_ids.extend(prev_ids);
            Ok(unseen_prev_ids)
        });

        let blocks = try_join_all(get_prev_ids_futs).await?.into_iter().flatten().collect();
        Ok(blocks)
    }
}

// TODO 
fn exec_with_retries() {
    
}


