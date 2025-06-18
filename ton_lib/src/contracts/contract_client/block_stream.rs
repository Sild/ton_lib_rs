use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::{TLClient, TLConnection};
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::block::block_id_ext::BlockIdExt;
use async_recursion::async_recursion;
use futures_util::future::try_join_all;
use std::collections::HashSet;
use std::time::Duration;

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
            let conn = stream.find_connection(from_seqno - 1).await?;
            let mc_block_id = conn.lookup_mc_block(from_seqno - 1).await?;
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

        let conn = self.find_connection(next_mc_seqno).await?;
        let mc_block_id = conn.lookup_mc_block(next_mc_seqno).await?;
        let shard_ids = conn.get_block_shards(mc_block_id.clone()).await?.shards;
        println!("shard for mc_seqno: {}: {:?}", mc_block_id.seqno, shard_ids);
        let unseen_shards = self.get_unseen_shards(conn, next_mc_seqno, shard_ids.clone()).await?;

        self.prev_mc_seqno = next_mc_seqno;
        self.prev_shards = shard_ids.into_iter().collect();

        let mut result: Vec<BlockIdExt> = unseen_shards.into_iter().collect();
        result.push(mc_block_id);
        Ok(Some(result))
    }

    #[async_recursion]
    async fn get_unseen_shards(
        &self,
        conn: &TLConnection,
        mc_seqno: u32,
        shard_ids: Vec<BlockIdExt>,
    ) -> Result<HashSet<BlockIdExt>, TonlibError> {
        let get_prev_ids_futs = shard_ids.into_iter().map(|block_id| async {
            if self.prev_shards.contains(&block_id) || block_id.seqno == 0 {
                return Ok::<_, TonlibError>(Default::default());
            }

            let prev_ids = self.get_prev_blocks_with_retry(conn, mc_seqno, &block_id).await?;
            let mut unseen_prev_ids = self.get_unseen_shards(conn, mc_seqno, prev_ids).await?;
            unseen_prev_ids.insert(block_id);
            Ok(unseen_prev_ids)
        });

        let blocks = try_join_all(get_prev_ids_futs).await?.into_iter().flatten().collect();
        Ok(blocks)
    }

    async fn find_connection(&self, mc_seqno: u32) -> Result<&TLConnection, TonlibError> {
        loop {
            let conn = self.client.get_connection();
            let mc_info = conn.get_mc_info().await?;
            if mc_info.last.seqno >= mc_seqno {
                return Ok(conn);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn get_prev_blocks_with_retry(
        &self,
        conn: &TLConnection,
        mc_seqno: u32,
        block_id: &BlockIdExt,
    ) -> Result<Vec<BlockIdExt>, TonlibError> {
        if let Ok(header) = conn.get_block_header(block_id.clone()).await {
            return Ok(header.prev_blocks.unwrap_or_default());
        }
        let mut last_error = None;
        for _ in 0..3 {
            let new_conn = self.find_connection(mc_seqno).await?;
            match new_conn.get_block_header(block_id.clone()).await {
                Ok(header) => return Ok(header.prev_blocks.unwrap_or_default()),
                Err(err) => last_error = Some(err),
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Err(last_error.unwrap())
    }
}
