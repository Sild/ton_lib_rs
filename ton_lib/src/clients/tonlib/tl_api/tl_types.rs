use crate::cell::ton_hash::ser_de::serde_ton_hash_b64;
use crate::cell::ton_hash::ser_de::serde_ton_hash_vec_b64;
use crate::clients::tonlib::tl_api::Base64Standard;
use crate::types::ton_address::ser_de::serde_ton_address_hex;
use std::borrow::Cow;
use std::fmt::Debug;

use crate::cell::ton_hash::TonHash;
use crate::types::ton_address::TonAddress;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

// tonlib_api.tl_api, line 23
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum TLKeyStoreType {
    #[serde(rename = "keyStoreTypeDirectory")]
    Directory { directory: String },
    #[serde(rename = "keyStoreTypeInMemory")]
    InMemory,
}

// tonlib_api.tl_api, line 26
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLConfig {
    #[serde(rename = "config")]
    pub net_config: String,
    pub blockchain_name: Option<String>,
    pub use_callbacks_for_network: bool,
    pub ignore_cache: bool,
}

// tonlib_api.tl_api, line 28
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLOptions {
    pub config: TLConfig,
    pub keystore_type: TLKeyStoreType,
}

// tonlib_api.tl_api, line 29
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename = "options.configInfo")]
pub struct TLOptionsConfigInfo {
    pub default_wallet_id: String,
    pub default_rwallet_init_public_key: String,
}

// tonlib_api.tl_api, line 30
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLOptionsInfo {
    pub config_info: TLOptionsConfigInfo,
}

// tonlib_api.tl_api, line 44
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLAccountAddress {
    #[serde(rename = "account_address", with = "serde_ton_address_hex")]
    pub address: TonAddress,
}

impl From<&TonAddress> for TLAccountAddress {
    fn from(address: &TonAddress) -> Self {
        TLAccountAddress {
            address: address.clone(),
        }
    }
}

impl From<TLAccountAddress> for TonAddress {
    fn from(tl_address: TLAccountAddress) -> Self { tl_address.address }
}

// tonlib_api.tl_api, line 48
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TLTxId {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
    #[serde(with = "serde_ton_hash_b64")]
    pub hash: TonHash,
}

impl TLTxId {
    pub const ZERO: TLTxId = TLTxId {
        lt: 0,
        hash: TonHash::ZERO,
    };
}

// tonlib_api.tl_api, line 50
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlockId {
    pub workchain: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub shard: i64,
    pub seqno: i32,
}

// tonlib_api.tl_api, line 51
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlockIdExt {
    pub workchain: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub shard: i64,
    pub seqno: i32,
    #[serde(with = "Base64Standard")]
    pub root_hash: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub file_hash: Vec<u8>,
}

// tonlib_api.tl_api, line 53
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRawFullAccountState {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: i64,
    #[serde(with = "Base64Standard")]
    pub code: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
    #[serde(rename = "last_transaction_id")]
    pub last_tx_id: TLTxId,
    pub block_id: TLBlockIdExt,
    #[serde(with = "Base64Standard")]
    pub frozen_hash: Vec<u8>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sync_utime: i64,
}

// tonlib_api.tl_api, line 54
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRawMessage {
    pub source: TLAccountAddress,
    pub destination: TLAccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub value: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub fwd_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ihr_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub created_lt: i64,
    #[serde(with = "Base64Standard")]
    pub body_hash: Vec<u8>,
    pub msg_data: TLMsgData,
}

// tonlib_api.tl_api, line 55
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRawTx {
    pub address: TLAccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub utime: i64,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
    #[serde(rename = "transaction_id")]
    pub tx_id: TLTxId,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub storage_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub other_fee: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_msg: Option<TLRawMessage>,
    pub out_msgs: Vec<TLRawMessage>,
}

// tonlib_api.tl_api, line 56
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRawTxs {
    #[serde(rename = "transactions")]
    pub txs: Vec<TLRawTx>,
    #[serde(rename = "previous_transaction_id")]
    pub prev_tx_id: TLTxId,
}
// tonlib_api.tl_api, line 58
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRawExtMessageInfo {
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
}

// tonlib_api.tl_api, line 60
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLPChanConfig {
    pub alice_public_key: String,
    pub alice_address: TLAccountAddress,
    pub bob_public_key: String,
    pub bob_address: TLAccountAddress,
    pub init_timeout: i32,
    pub close_timeout: i32,
    pub channel_id: i64,
}

// tonlib_api.tl_api, line 68
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRWalletLimit {
    pub seconds: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub value: i64,
}

// tonlib_api.tl_api, line 69
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRWalletConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub start_at: i64,
    pub limits: Vec<TLRWalletLimit>,
}

// tonlib_api.tl_api, line 75-81
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum TLAccountState {
    #[serde(rename = "raw.accountState")]
    Raw {
        #[serde(with = "Base64Standard")]
        code: Vec<u8>,
        #[serde(with = "Base64Standard")]
        data: Vec<u8>,
        #[serde(with = "Base64Standard")]
        frozen_hash: Vec<u8>,
    },
    #[serde(rename = "wallet.v3.accountState")]
    WalletV3 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "wallet.v4.accountState")]
    WalletV4 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "wallet.highload.v1.accountState")]
    WalletHighloadV1 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "wallet.highload.v2.accountState")]
    WalletHighloadV2 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
    },
    #[serde(rename = "dns.accountState")]
    DNS {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
    },
    #[serde(rename = "rwallet.accountState")]
    RWallet {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        unlocked_balance: i64,
        config: TLRWalletConfig,
    },
    #[serde(rename = "uninited.accountState")]
    Uninited {
        #[serde(with = "Base64Standard")]
        frozen_hash: Vec<u8>,
    },
    #[serde(rename = "pchan.accountState")]
    PChan {
        config: TLPChanConfig,
        state: TLPChanState,
        description: String,
    },
}

// tonlib_api.tl_api, line 83-85
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum TLPChanState {
    #[serde(rename = "pchan.stateInit")]
    Init {
        #[serde(rename = "signed_A")]
        signed_a: bool,
        #[serde(rename = "signed_B")]
        signed_b: bool,
        #[serde(rename = "min_A")]
        min_a: i64,
        #[serde(rename = "min_B")]
        min_b: i64,
        expire_at: i64,
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
    #[serde(rename = "pchan.stateClose")]
    Close {
        #[serde(rename = "signed_A")]
        signed_a: bool,
        #[serde(rename = "signed_B")]
        signed_b: bool,
        #[serde(rename = "min_A")]
        min_a: i64,
        #[serde(rename = "min_B")]
        min_b: i64,
        expire_at: i64,
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
    #[serde(rename = "pchan.statePayout")]
    Payout {
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
}

// tonlib_api.tl_api, line 90
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLFullAccountState {
    pub address: TLAccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: i64,
    #[serde(rename = "last_transaction_id")]
    pub last_tx_id: TLTxId,
    pub block_id: TLBlockIdExt,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sync_utime: i64,
    pub account_state: TLAccountState,
    pub revision: i32,
}

// tonlib_api.tl_api, line 95-96
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum TLSyncState {
    #[serde(rename = "syncStateDone")]
    Done,
    #[serde(rename = "syncStateInProgress")]
    InProgress {
        from_seqno: i32,
        to_seqno: i32,
        current_seqno: i32,
    },
}

// tonlib_api.tl_api, line 102-111
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum TLMsgData {
    #[serde(rename = "msg.dataRaw")]
    Raw {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
        #[serde(with = "Base64Standard")]
        init_state: Vec<u8>,
    },
    #[serde(rename = "msg.dataText")]
    Text {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
    #[serde(rename = "msg.dataDecryptedText")]
    DecryptedText {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
    #[serde(rename = "msg.dataEncryptedText")]
    EncryptedText {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
}

// tonlib_api.tl_api, line 179
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLSmcInfo {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
}

// tonlib_api.tl_api, line 181-182
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
pub enum TLSmcMethodId {
    #[serde(rename = "smc.methodIdNumber")]
    Number { number: i32 },
    #[serde(rename = "smc.methodIdName")]
    Name { name: Cow<'static, str> },
}

// tonlib_api.tl_api, line 184 - unsupported
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
// pub struct TLSmcRunResult {}

// tonlib_api.tl_api, line 186
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLSmcLibraryEntry {
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
}

// tonlib_api.tl_api, line 187
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLSmcLibraryResult {
    pub result: Vec<TLSmcLibraryEntry>,
}
// tonlib_api.tl_api, line 189
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename_all = "camelCase")]
pub enum TLSmcLibraryQueryExt {
    #[serde(rename = "smc.libraryQueryExt.one")]
    One {
        #[serde(with = "serde_ton_hash_b64")]
        hash: TonHash,
    },

    // tonlib_api.tl_api, line 190
    #[serde(rename = "smc.libraryQueryExt.scanBoc")]
    ScanBoc {
        #[serde(with = "Base64Standard")]
        boc: Vec<u8>,
        max_libs: i32,
    },
}
// tonlib_api.tl_api, line 191
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLSmcLibraryResultExt {
    #[serde(with = "Base64Standard")]
    pub dict_boc: Vec<u8>,
    #[serde(with = "serde_ton_hash_vec_b64")]
    pub libs_ok: Vec<TonHash>,
    #[serde(with = "serde_ton_hash_vec_b64")]
    pub libs_not_found: Vec<TonHash>,
}

// tonlib_api.tl_api, line 194
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLUpdateSyncState {
    pub sync_state: TLSyncState,
}

// tonlib_api.tl_api, line 209
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLLogVerbosityLevel {
    pub verbosity_level: u32,
}

// tonlib_api.tl_api, line 216
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLLiteServerInfo {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub now: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub version: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub capabilities: i64,
}

// tonlib_api.tl_api, line 219
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksMCInfo {
    pub last: TLBlockIdExt,
    #[serde(with = "Base64Standard")]
    pub state_root_hash: Vec<u8>,
    pub init: TLBlockIdExt,
}

// tonlib_api.tl_api, line 220
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksShards {
    pub shards: Vec<TLBlockIdExt>,
}

// tonlib_api.tl_api, line 221
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksAccountTxId {
    pub account: TonAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
}

// tonlib_api.tl_api, line 222
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksShortTxId {
    pub mode: u32,
    #[serde(with = "Base64Standard")]
    pub account: Vec<u8>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
}

// tonlib_api.tl_api, line 223
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksTxs {
    pub id: TLBlockIdExt,
    pub req_count: i32,
    pub incomplete: bool,
    #[serde(rename = "transactions")]
    pub txs: Vec<TLBlocksShortTxId>,
}

// tonlib_api.tl_api, line 224
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksTransactionsExt {
    pub id: TLBlockIdExt,
    pub req_count: i32,
    pub incomplete: bool,
    #[serde(rename = "transactions")]
    pub txs: Vec<TLRawTx>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TLConfigInfo {
    pub config: TLTvmCell,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TLTvmCell {
    #[serde(with = "Base64Standard")]
    pub bytes: Vec<u8>,
}

// tonlib_api.tl_api, line 225
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlocksHeader {
    pub id: TLBlockIdExt,
    pub global_id: i32,
    pub version: i32,
    pub flags: i32,
    pub after_merge: bool,
    pub after_split: bool,
    pub before_split: bool,
    pub want_merge: bool,
    pub want_split: bool,
    pub validator_list_hash_short: i32,
    pub catchain_seqno: i32,
    pub min_ref_mc_seqno: i32,
    pub is_key_block: bool,
    pub prev_key_block_seqno: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub start_lt: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub end_lt: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub gen_utime: i64,
    pub vert_seqno: Option<i32>,
    pub prev_blocks: Option<Vec<TLBlockIdExt>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TLUpdate {
    SyncState(TLUpdateSyncState),
}
