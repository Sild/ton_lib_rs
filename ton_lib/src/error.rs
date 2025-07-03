use hmac::digest::crypto_common;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::time::error::Elapsed;
use ton_lib_core::error::TLCoreError;
use ton_lib_core::types::{TonAddress, TxIdLTHash};
use ton_liteapi::tl::request::Request;
use ton_liteapi::types::LiteError;

#[macro_export]
macro_rules! bail_tl {
    ($($arg:tt)*) => {
        return Err(TLError::Custom(format!($($arg)*)))
    };
}

#[derive(Error, Debug)]
pub enum TLError {
    #[error("TLCoreError: {0}")]
    TLCoreError(#[from] TLCoreError),
    #[error("TLCoreError: {0}")]
    TLCoreArcError(#[from] Arc<TLCoreError>),
    #[error("NetRequestTimeout: {msg}, timeout={timeout:?}")]
    NetRequestTimeout { msg: String, timeout: Duration },

    // LiteClient
    #[error("LiteClientErrorResponse: {0:?}")]
    LiteClientErrorResponse(ton_liteapi::tl::response::Error),
    #[error("LiteClientWrongResponse: expected {0}, got {1}")]
    LiteClientWrongResponse(String, String),
    #[error("LiteClientLiteError: {0}")]
    LiteClientLiteError(#[from] LiteError),
    #[error("LiteClientConnTimeout: {0:?}")]
    LiteClientConnTimeout(Duration),
    #[error("LiteClientReqTimeout: {0:?}")]
    LiteClientReqTimeout(Box<(Request, Duration)>),

    // TonlibClient
    #[error("TLClientCreationFailed: tonlib_client_json_create returns null")]
    TLClientCreationFailed,
    #[error("TLClientWrongResponse: expected type: {0}, got: {1}")]
    TLClientWrongResponse(String, String),
    #[error("TLClientResponseError: code: {code}, message: {message}")]
    TLClientResponseError { code: i32, message: String },
    #[error("TLWrongArgs: {0}")]
    TLWrongArgs(String),
    #[error("TLSendError: fail to send request: {0}")]
    TLSendError(String),
    #[error("TLExecError: method: {method}, code: {code}, message: {message}")]
    TLExecError { method: String, code: i32, message: String },
    #[error("TLWrongUsage: {0}")]
    TLWrongUsage(String),

    // Emulators
    #[error("TVMEmulatorCreationFailed: emulator_create returns null")]
    EmulatorCreationFailed,
    #[error("TVMEmulatorSetFailed: fail to set param: {0}")]
    EmulatorSetParamFailed(&'static str),
    #[error("EmulatorNullResponse: emulator returns nullptr")]
    EmulatorNullResponse,
    #[error("TVMEmulatorResponseParseError: {field}, raw_response: {raw_response}")]
    EmulatorParseResponseError { field: &'static str, raw_response: String },
    #[error("EmulatorEmulationError: vm_exit_code: {vm_exit_code:?}, response_raw: {response_raw}")]
    EmulatorEmulationError {
        vm_exit_code: Option<i32>,
        response_raw: String,
    },

    // TVMStack
    #[error("TVMStackError: fail to pop specified type. expected: {0}, got: {1}")]
    TVMStackWrongType(String, String),
    #[error("TVMStackError: stack is empty")]
    TVMStackEmpty,

    // Mnemonic
    #[error("MnemonicWordsCount: expected 24 words, got {0}")]
    MnemonicWordsCount(usize),
    #[error("MnemonicWord: unexpected word {0}")]
    MnemonicWord(String),
    #[error("MnemonicFirstByte: first byte can't be {0}")]
    MnemonicFirstByte(u8),
    #[error("MnemonicFirstBytePassless: first byte can't be {0}")]
    MnemonicFirstBytePassless(u8),

    // General errors
    #[error("UnexpectedValue: expected: {expected}, actual: {actual}")]
    UnexpectedValue { expected: String, actual: String },

    // TonActiveContract
    #[error("TonContractNoData: contract {address} has no data at tx_id {tx_id:?}")]
    TonContractNoData {
        address: TonAddress,
        tx_id: Option<TxIdLTHash>,
    },
    #[error("CustomError: {0}")]
    Custom(String),

    #[error("{0}")]
    HmacInvalidLen(#[from] crypto_common::InvalidLength),
    #[error("{0}")]
    NullError(#[from] std::ffi::NulError),
    #[error("{0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("{0}")]
    UTF8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    FromHexError(#[from] hex::FromHexError),
    #[error("{0}")]
    ElapsedError(#[from] Elapsed),
    #[error("{0}")]
    AdnlError(#[from] adnl::AdnlError),

    // handling external errors
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("{0}")]
    RecvError(#[from] tokio::sync::oneshot::error::RecvError),
}

impl From<TLError> for TLCoreError {
    fn from(err: TLError) -> Self { TLCoreError::Custom(err.to_string()) }
}

impl From<&TLError> for TLCoreError {
    fn from(err: &TLError) -> Self { TLCoreError::Custom(err.to_string()) }
}
