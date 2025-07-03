use crate::clients::block_stream::BlockStream;
use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::TLClient;
use crate::contracts::tl_provider::cache_stats::CacheStats;
use crate::contracts::tl_provider::provider_config::TLProviderConfig;
use crate::emulators::tvm::tvm_response::TVMGetMethodSuccess;
use crate::error::TLError;
use futures_util::future::{join_all, try_join_all};
use moka::future::Cache;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};
use std::time::Duration;
use ton_lib_core::cell::TonHash;
use ton_lib_core::error::TLCoreError;
use ton_lib_core::traits::contract_provider::ContractState;
use ton_lib_core::types::{TonAddress, TxIdLTHash};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EmulateGetMethodArgs {
    pub state: Arc<ContractState>,
    pub method_id: i32,
    pub stack: Option<Arc<Vec<u8>>>,
}

impl EmulateGetMethodArgs {
    pub fn new(state: Arc<ContractState>, method_id: i32, stack: Option<Arc<Vec<u8>>>) -> Self {
        Self {
            state,
            method_id,
            stack,
        }
    }
}

pub(crate) struct StateCache {
    pub tl_client: TLClient,
    pub latest_tx_cache: Cache<TonAddress, TxIdLTHash>,
    pub state_latest_cache: Cache<TonAddress, Arc<ContractState>>,
    pub state_by_tx_cache: Cache<TxIdLTHash, Arc<ContractState>>,
    pub emulate_get_method_cache: Cache<EmulateGetMethodArgs, TVMGetMethodSuccess>,
    pub cache_stats: CacheStats,
}

impl StateCache {
    pub(crate) async fn new(config: TLProviderConfig, tl_client: TLClient) -> Result<Arc<Self>, TLError> {
        let block_stream = BlockStream::new(tl_client.clone(), config.stream_from_seqno, None).await?;
        let (capacity, ttl) = (config.cache_capacity, config.cache_ttl);
        let cache = Arc::new(Self {
            tl_client,
            latest_tx_cache: init_cache(capacity, ttl),
            state_latest_cache: init_cache(capacity, ttl),
            state_by_tx_cache: init_cache(capacity, ttl),
            emulate_get_method_cache: init_cache(capacity, ttl),
            cache_stats: CacheStats::default(),
        });
        let weak = Arc::downgrade(&cache);
        tokio::spawn(recent_tx_loop(weak, block_stream, config.idle_on_error));
        Ok(cache)
    }

    pub(crate) async fn get_latest(&self, address: &TonAddress) -> Result<Arc<ContractState>, TLError> {
        self.cache_stats.state_latest_req.fetch_add(1, Relaxed);
        let state = if let Some(id) = self.latest_tx_cache.get(address).await {
            self.state_latest_cache.try_get_with_by_ref(address, self.load_contract(address, Some(&id))).await?
        } else {
            self.state_latest_cache.try_get_with_by_ref(address, self.load_contract(address, None)).await?
        };
        Ok(state)
    }

    pub(crate) async fn get_by_tx(
        &self,
        address: &TonAddress,
        tx_id: &TxIdLTHash,
    ) -> Result<Arc<ContractState>, TLError> {
        self.cache_stats.state_by_tx_req.fetch_add(1, Relaxed);
        Ok(self.state_by_tx_cache.try_get_with_by_ref(tx_id, self.load_contract(address, Some(tx_id))).await?)
    }

    async fn load_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxIdLTHash>,
    ) -> Result<Arc<ContractState>, TLCoreError> {
        let account_state = match tx_id {
            Some(tx_id) => {
                self.cache_stats.state_by_tx_miss.fetch_add(1, Relaxed);
                self.tl_client.get_account_state_raw_by_tx(address.clone(), tx_id.clone()).await?
            }
            None => {
                self.cache_stats.state_latest_miss.fetch_add(1, Relaxed);
                self.tl_client.get_account_state_raw(address.clone()).await?
            }
        };
        let code_boc = match account_state.code.is_empty() {
            true => None,
            false => Some(account_state.code),
        };

        let data_boc = match account_state.data.is_empty() {
            true => None,
            false => Some(account_state.data),
        };

        let frozen_hash = match account_state.frozen_hash.is_empty() {
            true => None,
            false => Some(TonHash::from_vec(account_state.frozen_hash)?),
        };

        Ok(Arc::new(ContractState {
            address: address.clone(),
            mc_seqno: account_state.block_id.seqno,
            last_tx_id: account_state.last_tx_id,
            code_boc,
            data_boc,
            frozen_hash,
            balance: account_state.balance,
        }))
    }
}

async fn recent_tx_loop(weak_inner: Weak<StateCache>, mut stream: BlockStream, idle_on_error: Duration) {
    log::info!("[recent_tx_loop] started with last_mc_seqno: {}", stream.last_mc_seqno());

    loop {
        let inner = match weak_inner.upgrade() {
            Some(inner) => inner,
            None => {
                log::warn!("[recent_tx_loop] inner is dropped");
                break;
            }
        };
        let inner_ref = &inner;

        let block_ids = match stream.next().await {
            Ok(Some(block_ids)) => block_ids,
            Ok(None) => {
                log::warn!("[tx_update_loop] got None from stream");
                break;
            }
            Err(err) => {
                log::warn!("[tx_update_loop] error getting next block: {err:?}");
                tokio::time::sleep(idle_on_error).await;
                continue;
            }
        };

        let txs_by_addr_futs = block_ids.into_iter().map(|block_id| async move {
            let txs = inner_ref.tl_client.get_block_txs(&block_id).await?;
            let mut txs_by_addr = HashMap::new();
            for tx in txs {
                let address = TonAddress::new(block_id.shard_ident.workchain, tx.address_hash);
                // in shard scope we always need the latest tx
                txs_by_addr.insert(address, TxIdLTHash::new(tx.lt, tx.tx_hash));
            }
            Ok::<_, TLError>(txs_by_addr)
        });
        let txs_by_addr_vec = match try_join_all(txs_by_addr_futs).await {
            Ok(res) => res,
            Err(err) => {
                log::warn!("[recent_tx_loop] error getting shards_txs: {err:?}");
                tokio::time::sleep(idle_on_error).await;
                continue;
            }
        };

        let mut txs_by_addr_latest = HashMap::<TonAddress, TxIdLTHash>::with_capacity(txs_by_addr_vec.len());
        for txs_by_addr in txs_by_addr_vec {
            for (address, tx_id) in txs_by_addr {
                if let Some(cur_tx_id) = txs_by_addr_latest.get(&address) {
                    if cur_tx_id.lt < tx_id.lt {
                        txs_by_addr_latest.insert(address, tx_id);
                    }
                } else {
                    txs_by_addr_latest.insert(address, tx_id);
                }
            }
        }

        let update_cache_futs = txs_by_addr_latest.into_iter().map(|(address, tx_id)| async move {
            inner_ref.latest_tx_cache.insert(address.clone(), tx_id).await;
            inner_ref.state_latest_cache.invalidate(&address).await;
        });
        join_all(update_cache_futs).await;
    }
    log::info!("[recent_tx_loop] completed");
}

fn init_cache<K, V>(capacity: u64, ttl: Duration) -> Cache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    Cache::builder().max_capacity(capacity).time_to_live(ttl).build()
}
