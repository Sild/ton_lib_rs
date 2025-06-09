use crate::clients::client_types::TxId;
use crate::contracts::contract_client::data_provider::DataProvider;
use crate::contracts::contract_client::types::ContractState;
use crate::errors::TonlibError;
use crate::types::ton_address::TonAddress;
use moka::future::Cache;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

pub struct ContractClientCacheConfig {
    pub loop_error_sleep_duration: Duration,
    pub tx_ids_capacity: u64,
    pub tx_ids_ttl: Duration,
    pub states_capacity: u64,
    pub states_ttl: Duration,
}

pub struct ContractClientCacheStats {
    pub state_latest_req: usize,
    pub state_latest_miss: usize,
    pub state_by_tx_req: usize,
    pub state_by_tx_miss: usize,
}

/// Except regular cache mechanic, maintain latest tx_id for each address
/// Invalidate state cache if new tx_id is received
pub(super) struct ContractClientCache {
    inner: Arc<Inner>,
}

impl ContractClientCache {
    pub(crate) fn new(
        config: ContractClientCacheConfig,
        data_provider: Arc<dyn DataProvider>,
        start_mc_seqno: u32,
    ) -> Result<Self, TonlibError> {
        let loop_error_sleep_duration = config.loop_error_sleep_duration;
        let inner = Arc::new(Inner::new(config, data_provider.clone()));
        let weak = Arc::downgrade(&inner);
        tokio::spawn(latest_tx_updater_loop(weak, data_provider, start_mc_seqno, loop_error_sleep_duration));
        Ok(Self { inner })
    }

    pub(crate) async fn get_state(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxId>,
    ) -> Result<Arc<ContractState>, TonlibError> {
        let load_arc_state = async |tx_id_int: Option<&TxId>| {
            match &tx_id_int {
                Some(_) => self.inner.state_by_tx_miss.fetch_add(1, Ordering::Relaxed),
                None => self.inner.state_latest_req.fetch_add(1, Ordering::Relaxed),
            };
            self.inner.data_provider.get_state(address, tx_id_int).await.map(Arc::new)
        };

        match &tx_id {
            Some(_) => self.inner.state_latest_req.fetch_add(1, Ordering::Relaxed),
            None => self.inner.state_latest_miss.fetch_add(1, Ordering::Relaxed),
        };

        if let Some(tx_id) = tx_id {
            return match self.inner.latest_states.get(address).await {
                Some(state) if &state.last_tx_id == tx_id => Ok(state.clone()),
                _ => load_arc_state(Some(tx_id)).await,
            };
        }

        let recent_tx = self.inner.latest_txs.get(address).await;
        Ok(self.inner.latest_states.try_get_with(address.clone(), load_arc_state(recent_tx.as_ref())).await?)
    }

    pub(crate) fn get_stats(&self) -> ContractClientCacheStats {
        ContractClientCacheStats {
            state_latest_req: self.inner.state_latest_req.load(Ordering::Relaxed),
            state_latest_miss: self.inner.state_latest_miss.load(Ordering::Relaxed),
            state_by_tx_req: self.inner.state_by_tx_req.load(Ordering::Relaxed),
            state_by_tx_miss: self.inner.state_by_tx_miss.load(Ordering::Relaxed),
        }
    }
}

async fn latest_tx_updater_loop(
    weak: Weak<Inner>,
    data_provider: Arc<dyn DataProvider>,
    start_mc_seqno: u32,
    error_sleep_duration: Duration,
) -> Result<(), TonlibError> {
    log::info!("[ContractClientCache][run_loop] started with mc_seqno: {start_mc_seqno}");
    let mut cur_mc_seqno = start_mc_seqno;
    loop {
        let tx_ids = match data_provider.get_latest_txs(cur_mc_seqno).await {
            Ok(tx_ids) => tx_ids,
            Err(err) => {
                log::error!(
                    "[ContractClientCache][run_loop]: fail to call get_latest_txs for mc_seqno={cur_mc_seqno}: {err}"
                );
                tokio::time::sleep(error_sleep_duration).await;
                continue;
            }
        };
        let Some(cache_data) = weak.upgrade() else { break };

        for (address, tx_id) in tx_ids {
            cache_data.latest_states.invalidate(&address).await;
            cache_data.latest_txs.insert(address, tx_id).await;
        }
        cur_mc_seqno += 1;
    }
    log::info!("[ContractClientCache] run_loop completed: cache_data is dropped");
    Ok(())
}

struct Inner {
    data_provider: Arc<dyn DataProvider>,
    latest_txs: Cache<TonAddress, TxId>,
    latest_states: Cache<TonAddress, Arc<ContractState>>,
    state_latest_req: AtomicUsize,
    state_latest_miss: AtomicUsize,
    state_by_tx_req: AtomicUsize,
    state_by_tx_miss: AtomicUsize,
}

impl Inner {
    fn new(config: ContractClientCacheConfig, data_provider: Arc<dyn DataProvider>) -> Self {
        let latest_tx_ids =
            Cache::builder().max_capacity(config.tx_ids_capacity).time_to_live(config.tx_ids_ttl).build();

        let latest_states =
            Cache::builder().max_capacity(config.states_capacity).time_to_live(config.states_ttl).build();

        Self {
            data_provider,
            latest_txs: latest_tx_ids,
            latest_states,
            state_latest_req: AtomicUsize::new(0),
            state_latest_miss: AtomicUsize::new(0),
            state_by_tx_req: AtomicUsize::new(0),
            state_by_tx_miss: AtomicUsize::new(0),
        }
    }
}

impl Default for ContractClientCacheConfig {
    fn default() -> Self {
        Self {
            loop_error_sleep_duration: Duration::from_millis(100),
            tx_ids_capacity: 1000,
            tx_ids_ttl: Duration::from_secs(300), // 5 min
            states_capacity: 1000,
            states_ttl: Duration::from_secs(300), // 5 min
        }
    }
}
