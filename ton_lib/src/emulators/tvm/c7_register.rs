use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use crate::types::ton_address::TonAddress;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct TVMEmulatorC7 {
    pub address: TonAddress,
    pub unix_time: u32,
    pub balance: u64,
    pub rand_seed: TonHash,
    pub config: Vec<u8>,
}

impl TVMEmulatorC7 {
    pub fn new(address: TonAddress, config: Vec<u8>) -> Result<Self, TonlibError> {
        Ok(Self {
            address,
            unix_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,
            balance: 0,
            rand_seed: TonHash::ZERO,
            config,
        })
    }
}
