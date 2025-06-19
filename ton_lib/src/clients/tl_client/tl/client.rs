use crate::block_tlb::BlockIdExt;
use crate::clients::tl_client::connection::TLConnection;
use crate::clients::tl_client::tl::request::TLRequest;
use crate::clients::tl_client::tl::response::TLResponse;
use crate::clients::tl_client::tl::types::{
    TLBlockId, TLBlocksAccountTxId, TLBlocksHeader, TLBlocksMCInfo, TLBlocksShards, TLBlocksTxs, TLFullAccountState,
    TLRawFullAccountState, TLRawTxs, TLTxId,
};
use crate::clients::tl_client::RetryStrategy;
use crate::error::TLError;
use crate::libs_dict::LibsDict;
use crate::unwrap_tl_response;
use async_trait::async_trait;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::RetryIf;
use ton_lib_core::cell::{TonCellRef, TonHash};
use ton_lib_core::constants::{TON_MC_ID, TON_SHARD_FULL};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxIdLTAddress};

#[async_trait]
pub trait TLClientTrait: Send + Sync {
    fn get_connection(&self) -> &TLConnection;

    async fn exec(&self, req: &TLRequest) -> Result<TLResponse, TLError> {
        let retry_strat = self.get_retry_strategy();
        let fi = FixedInterval::new(retry_strat.retry_waiting);
        let strategy = fi.take(retry_strat.retry_count);
        RetryIf::spawn(strategy, || async { self.get_connection().exec_impl(req).await }, retry_condition).await
    }

    async fn get_mc_info(&self) -> Result<TLBlocksMCInfo, TLError> {
        let req = TLRequest::BlocksGetMCInfo {};
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksMCInfo)
    }

    /// * `mode`: Lookup mode: `1` - by `block_id.seqno`, `2` - by `lt`, `4` - by `utime`.
    async fn lookup_block(&self, mode: i32, block_id: TLBlockId, lt: i64, utime: i32) -> Result<BlockIdExt, TLError> {
        let req = TLRequest::BlocksLookupBlock {
            mode,
            id: block_id,
            lt,
            utime,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLBlockIdExt)
    }

    async fn lookup_mc_block(&self, seqno: u32) -> Result<BlockIdExt, TLError> {
        let block_id = TLBlockId {
            workchain: TON_MC_ID,
            shard: TON_SHARD_FULL as i64,
            seqno: seqno as i32,
        };
        self.lookup_block(1, block_id, 0, 0).await
    }

    async fn get_account_state(&self, address: TonAddress) -> Result<TLFullAccountState, TLError> {
        let req = TLRequest::GetAccountState {
            account_address: address.into(),
        };
        Ok(*unwrap_tl_response!(self.exec(&req).await?, TLFullAccountState)?)
    }

    async fn get_account_state_raw(&self, address: TonAddress) -> Result<TLRawFullAccountState, TLError> {
        let req = TLRequest::RawGetAccountState {
            account_address: address.into(),
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawFullAccountState)
    }

    async fn get_account_state_raw_by_tx(
        &self,
        address: TonAddress,
        tx_id: TLTxId,
    ) -> Result<TLRawFullAccountState, TLError> {
        let req = TLRequest::RawGetAccountStateByTx {
            account_address: address.into(),
            transaction_id: tx_id,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawFullAccountState)
    }

    async fn get_txs(&self, address: TonAddress, from_tx: TLTxId) -> Result<TLRawTxs, TLError> {
        let req = TLRequest::RawGetTxs {
            account_address: address.into(),
            from_transaction_id: from_tx,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawTxs)
    }

    async fn get_txs_v2(
        &self,
        address: TonAddress,
        from_tx: TLTxId,
        count: usize,
        try_decode_msg: bool,
    ) -> Result<TLRawTxs, TLError> {
        if count > 16 {
            return Err(TLError::TLWrongArgs(format!("get_raw_transactions_v2: count <= 16 supported, got {count}")));
        }
        let req = TLRequest::RawGetTxsV2 {
            account_address: address.into(),
            from_transaction_id: from_tx.clone(),
            count: count as u32,
            try_decode_messages: try_decode_msg,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawTxs)
    }

    async fn send_msg(&self, body: Vec<u8>) -> Result<TonHash, TLError> {
        let req = TLRequest::RawSendMsgReturnHash { body };
        let rsp = unwrap_tl_response!(self.exec(&req).await?, TLRawExtMessageInfo)?;
        Ok(TonHash::from_vec(rsp.hash)?)
    }

    async fn sync(&self) -> Result<BlockIdExt, TLError> {
        let req = TLRequest::Sync {};
        unwrap_tl_response!(self.exec(&req).await?, TLBlockIdExt)
    }

    /// May return less libraries when requested
    /// Check it on user side if you need it
    /// If no libraries found, returns None
    async fn get_libs(&self, lib_ids: Vec<TonHash>) -> Result<Option<LibsDict>, TLError> {
        let req = TLRequest::SmcGetLibraries { library_list: lib_ids };
        let result = unwrap_tl_response!(self.exec(&req).await?, TLSmcLibraryResult)?;
        if result.result.is_empty() {
            return Ok(None);
        }
        let mut libs_dict = LibsDict::default();
        for lib in result.result {
            libs_dict.insert(TonHash::from_vec(lib.hash)?, TonCellRef::from_boc(&lib.data)?);
        }
        Ok(Some(libs_dict))
    }

    async fn get_block_shards(&self, block_id: BlockIdExt) -> Result<TLBlocksShards, TLError> {
        let req = TLRequest::BlocksGetShards { id: block_id };
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksShards)
    }

    /// Returns up to specified number of ids of transactions in specified block.
    ///
    /// * `block_id`: ID of the block to retrieve transactions for (either masterchain or shard).
    /// * `mode`: Use `7` to get first chunk of transactions or `7 + 128` for subsequent chunks.
    /// * `count`: Maximum mumber of transactions to retrieve.
    /// * `after`: Specify `NULL_BLOCKS_ACCOUNT_TRANSACTION_ID` to get the first chunk
    ///             or id of the last retrieved tx for subsequent chunks.
    ///
    async fn get_block_txs(
        &self,
        block_id: BlockIdExt,
        mode: u32,
        count: u32,
        after: TxIdLTAddress,
    ) -> Result<TLBlocksTxs, TLError> {
        let req = TLRequest::BlocksGetTxs {
            id: block_id,
            mode,
            count,
            after: TLBlocksAccountTxId {
                account: after.address,
                lt: after.lt,
            },
        };
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksTxs)
    }

    async fn get_block_header(&self, block_id: BlockIdExt) -> Result<TLBlocksHeader, TLError> {
        let req = TLRequest::GetBlockHeader { id: block_id };
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksHeader)
    }

    async fn get_config_boc_param(&self, mode: u32, param: u32) -> Result<Vec<u8>, TLError> {
        let req = TLRequest::GetConfigParam { mode, param };
        Ok(unwrap_tl_response!(self.exec(&req).await?, TLConfigInfo)?.config.bytes)
    }
    // TODO find out about mode. Use 0 by default - it works well
    async fn get_config_boc_all(&self, mode: u32) -> Result<Vec<u8>, TLError> {
        let req = TLRequest::GetConfigAll { mode };
        Ok(unwrap_tl_response!(self.exec(&req).await?, TLConfigInfo)?.config.bytes)
    }

    fn get_retry_strategy(&self) -> &RetryStrategy;
}

fn retry_condition(error: &TLError) -> bool {
    match error {
        TLError::TLClientResponseError { code, .. } => *code == 500,
        _ => false,
    }
}
