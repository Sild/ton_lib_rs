use crate::emulators::emul_utils::require_field;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::account::ShardAccount;
use crate::types::tlb::block_tlb::tx::Tx;
use crate::types::tlb::TLB;
use base64::prelude::BASE64_STANDARD;
use base64_serde::base64_serde_type;
use serde::Deserialize;

base64_serde_type!(Base64Standard, BASE64_STANDARD);

#[derive(Debug, Clone)]
pub struct TXEmulationSuccess {
    pub success: bool,
    pub tx_boc_b64: String,
    pub shard_account_boc_b64: String,
    pub vm_log: String,
    pub actions: Option<String>,
    pub elapsed_time: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TXEmulationResponse {
    pub success: bool,
    #[serde(rename = "transaction")]
    pub tx_boc_b64: Option<String>,
    #[serde(rename = "shard_account")]
    pub shard_account_boc_b64: Option<String>,
    pub vm_log: Option<String>,
    pub actions: Option<String>,
    pub elapsed_time: f64,
    pub error: Option<String>,
    pub external_not_accepted: Option<bool>,
    pub vm_exit_code: Option<i32>,
    #[serde(skip)]
    pub raw_response: String,
}

impl TXEmulationResponse {
    pub fn from_json(json: String) -> Result<Self, TonlibError> {
        let mut value: Self = serde_json::from_str(&json)?;
        value.raw_response = json;
        Ok(value)
    }

    pub fn into_success(self) -> Result<TXEmulationSuccess, TonlibError> {
        if !self.success {
            return Err(TonlibError::EmulatorEmulationError {
                vm_exit_code: self.vm_exit_code,
                response_raw: self.raw_response,
            });
        }
        let tx_boc_b64 = require_field(self.tx_boc_b64, "tx_boc", &self.raw_response)?;
        let shard_account_boc_b64 = require_field(self.shard_account_boc_b64, "shard_account_boc", &self.raw_response)?;
        let vm_log = self.vm_log.unwrap_or_default();
        Ok(TXEmulationSuccess {
            success: self.success,
            tx_boc_b64,
            shard_account_boc_b64,
            vm_log,
            actions: self.actions,
            elapsed_time: self.elapsed_time,
        })
    }
}

impl TXEmulationSuccess {
    pub fn shard_account_parsed(&self) -> Result<ShardAccount, TonlibError> {
        ShardAccount::from_boc_base64(&self.shard_account_boc_b64)
    }
    pub fn tx_parsed(&self) -> Result<Tx, TonlibError> { Tx::from_boc_base64(&self.tx_boc_b64) }
}
