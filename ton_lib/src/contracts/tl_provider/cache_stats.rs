use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Default)]
pub(crate) struct CacheStats {
    pub state_latest_req: AtomicUsize,
    pub state_latest_miss: AtomicUsize,
    pub state_by_tx_req: AtomicUsize,
    pub state_by_tx_miss: AtomicUsize,
    pub emulate_get_method_req: AtomicUsize,
    pub emulate_get_method_miss: AtomicUsize,
}

impl CacheStats {
    pub(crate) fn export(
        &self,
        latest_entry_count: usize,
        by_tx_entry_count: usize,
        emulate_get_method_entry_count: usize,
    ) -> HashMap<String, usize> {
        HashMap::from([
            ("state_latest_req".to_string(), self.state_latest_req.load(Relaxed)),
            ("state_latest_miss".to_string(), self.state_latest_miss.load(Relaxed)),
            ("state_latest_entry_count".to_string(), latest_entry_count),
            ("state_by_tx_req".to_string(), self.state_by_tx_req.load(Relaxed)),
            ("state_by_tx_miss".to_string(), self.state_by_tx_miss.load(Relaxed)),
            ("state_by_tx_entry_count".to_string(), by_tx_entry_count),
            ("emulate_get_method_req".to_string(), self.emulate_get_method_req.load(Relaxed)),
            ("emulate_get_method_miss".to_string(), self.emulate_get_method_miss.load(Relaxed)),
            ("emulate_get_method_entry_count".to_string(), emulate_get_method_entry_count),
        ])
    }
}
