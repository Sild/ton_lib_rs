use crate::cell::ton_hash::ser_de::serde_ton_hash_vec_b64;
use crate::cell::ton_hash::TonHash;
use crate::clients::tonlib::tl_api::tl_types::{
    TLAccountAddress, TLBlockId, TLBlockIdExt, TLBlocksAccountTxId, TLOptions, TLSmcLibraryQueryExt, TLTxId,
};
use crate::clients::tonlib::tl_api::Base64Standard;
use crate::errors::TonlibError;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::CString;
use strum::IntoStaticStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TonLibraryId {
    pub id: Vec<u8>,
}

impl Serialize for TonLibraryId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(&self.id))
    }
}

impl<'de> Deserialize<'de> for TonLibraryId {
    fn deserialize<D>(deserializer: D) -> Result<TonLibraryId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        // Try to decode the base64 string
        let bytes = match STANDARD.decode(s) {
            Ok(decoded) => decoded,
            Err(_) => return Err(serde::de::Error::custom("Invalid base64 string")),
        };

        Ok(TonLibraryId { id: bytes })
    }
}

impl From<&TonHash> for TonLibraryId {
    fn from(value: &TonHash) -> Self {
        TonLibraryId {
            id: value.as_slice().to_vec(),
        }
    }
}

impl TryFrom<TonLibraryId> for TonHash {
    type Error = TonlibError;

    fn try_from(value: TonLibraryId) -> Result<Self, Self::Error> { TonHash::from_vec(value.id) }
}

#[derive(IntoStaticStr, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename_all = "camelCase")]
pub enum TLRequest {
    // tonlib_api.tl, line 216
    LiteServerInfo {
        now: i64,
        version: i32,
        capabilities: i64,
    },

    // tonlib_api.tl, line 238
    Init {
        options: TLOptions,
    },

    // tonlib_api.tl, line 261
    #[serde(rename = "raw.sendMessageReturnHash")]
    RawSendMsgReturnHash {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
    },

    // tonlib_api.tl, line 265
    #[serde(rename = "sync")]
    Sync {},

    //tonlib_api.tl, line 266
    #[serde(rename = "raw.getAccountState")]
    RawGetAccountState {
        account_address: TLAccountAddress,
    },

    //tonlib_api.tl, line 267
    #[serde(rename = "raw.getAccountStateByTransaction")]
    RawGetAccountStateByTx {
        account_address: TLAccountAddress,
        transaction_id: TLTxId,
    },

    // tonlib_api.tl, line 268
    #[serde(rename = "raw.getTransactions")]
    RawGetTxs {
        account_address: TLAccountAddress,
        from_transaction_id: TLTxId,
    },

    // tonlib_api.tl, line 269
    #[serde(rename = "raw.getTransactionsV2")]
    RawGetTxsV2 {
        account_address: TLAccountAddress,
        from_transaction_id: TLTxId,
        count: u32,
        try_decode_messages: bool,
    },

    // tonlib_api.tl, line 270
    #[serde(rename = "raw.sendMessage")]
    RawSendMessage {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
    },

    // tonlib_api.tl, line 288
    #[serde(rename = "getAccountState")]
    GetAccountState {
        account_address: TLAccountAddress,
    },

    // tonlib_api.tl, line 294
    #[serde(rename = "getConfigParam")]
    GetConfigParam {
        mode: u32,
        param: u32,
    },

    // tonlib_api.tl, line 295
    #[serde(rename = "getConfigAll")]
    GetConfigAll {
        mode: u32,
    },

    // tonlib_api.tl, line 306
    #[serde(rename = "smc.load")]
    SmcLoad {
        account_address: TLAccountAddress,
    },

    // tonlib_api.tl, line 307
    #[serde(rename = "smc.loadByTransaction")]
    SmcLoadByTransaction {
        account_address: TLAccountAddress,
        transaction_id: TLTxId,
    },

    // tonlib_api.tl, line 308
    #[serde(rename = "smc.forget")]
    SmcForget {
        id: i64,
    },

    // tonlib_api.tl, line 309
    #[serde(rename = "smc.getCode")]
    SmcGetCode {
        id: i64,
    },

    // tonlib_api.tl, line 310
    #[serde(rename = "smc.getData")]
    SmcGetData {
        id: i64,
    },

    // tonlib_api.tl, line 311
    #[serde(rename = "smc.getState")]
    SmcGetState {
        id: i64,
    },

    // tonlib_api.tl, line 312
    #[serde(rename = "smc.runGetMethod")]
    SmcRunGetMethod(()), // Unsupported

    // tonlib_api.tl, line 314
    #[serde(rename = "smc.getLibraries")]
    SmcGetLibraries {
        #[serde(with = "serde_ton_hash_vec_b64")]
        library_list: Vec<TonHash>,
    },

    // tonlib_api.tl, line 315
    #[serde(rename = "smc.getLibrariesExt")]
    SmcGetLibrariesExt {
        list: Vec<TLSmcLibraryQueryExt>,
    },

    // tonlib_api.tl, line 316
    #[serde(rename = "blocks.getMasterchainInfo")]
    BlocksGetMCInfo {},

    // tonlib_api.tl, line 327
    #[serde(rename = "blocks.getShards")]
    BlocksGetShards {
        id: TLBlockIdExt,
    },

    // tonlib_api.tl, line 328
    #[serde(rename = "blocks.lookupBlock")]
    BlocksLookupBlock {
        mode: i32,
        id: TLBlockId,
        lt: i64,
        utime: i32,
    },

    // tonlib_api.tl, line 329
    #[serde(rename = "blocks.getTransactions")]
    BlocksGetTxs {
        id: TLBlockIdExt,
        mode: u32,
        count: u32,
        after: TLBlocksAccountTxId,
    },

    // tonlib_api.tl, line 330
    #[serde(rename = "blocks.getTransactionsExt")]
    BlocksGetTransactionsExt {
        id: TLBlockIdExt,
        mode: u32,
        count: u32,
        after: TLBlocksAccountTxId,
    },

    // tonlib_api.tl, line 331
    #[serde(rename = "blocks.getBlockHeader")]
    GetBlockHeader {
        id: TLBlockIdExt,
    },

    // tonlib_ai.tl, line 342
    #[serde(rename = "liteServer.getInfo")]
    LiteServerGetInfo {},

    // tonlib_api.tl, line 352
    SetLogVerbosityLevel {
        new_verbosity_level: u32,
    },
    // tonlib_api.tl, line 355
    GetLogVerbosityLevel {},
}

impl TLRequest {
    pub fn to_c_str_json(&self, extra: &str) -> Result<CString, TonlibError> {
        let mut value = serde_json::to_value(self)?;
        let obj = value.as_object_mut().unwrap();
        let extra_val = serde_json::Value::from(extra);
        obj.insert(String::from("@extra"), extra_val);
        Ok(CString::new(serde_json::to_string(&value)?)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::clients::tonlib::tl_api::tl_request::TLRequest;
    use std::ffi::CString;

    #[test]
    fn test_tl_request_to_c_str_json() -> anyhow::Result<()> {
        let req = TLRequest::SetLogVerbosityLevel {
            new_verbosity_level: 100500,
        };
        let cstr: CString = req.to_c_str_json("some_extra")?;
        assert_eq!(
            "{\"@extra\":\"some_extra\",\"@type\":\"setLogVerbosityLevel\",\"new_verbosity_level\":100500}",
            cstr.to_str()?
        );
        Ok(())
    }
}
