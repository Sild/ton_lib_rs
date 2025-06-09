use crate::clients::client_types::TxId;
use crate::types::ton_address::TonAddress;
use hex::FromHexError;
use hmac::digest::crypto_common;
use num_bigint::BigUint;
use std::env::VarError;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use ton_liteapi::tl::request::Request;
use ton_liteapi::types::LiteError;

#[derive(Error, Debug)]
pub enum TonlibError {
    // ton_hash
    #[error("TonHashWrongLen: Expecting {exp} bytes, got {given}")]
    TonHashWrongLen { exp: usize, given: usize },
    #[error("TonAddressParseError: address={0}, err: {1}")]
    TonAddressParseError(String, String),

    // cell_parser
    #[error("ParserDataUnderflow: Requested {req} bits, but only {left} left")]
    ParserDataUnderflow { req: usize, left: usize },
    #[error("ParserBadPosition: New position is {new_pos}, but data_bits_len is {bits_len}")]
    ParserBadPosition { new_pos: i32, bits_len: usize },
    #[error("ParserWrongSlicePosition: expecting bit_pos=0, next_ref_pos=0. Got bit_position={bit_pos}, next_ref_position={next_ref_pos}")]
    ParserWrongSlicePosition { bit_pos: usize, next_ref_pos: usize },
    #[error("ParserRefsUnderflow: No ref with index={req}")]
    ParserRefsUnderflow { req: usize },
    #[error("ParserCellNotEmpty: Cell is not empty: {bits_left} bits left, {refs_left} refs left")]
    ParserCellNotEmpty { bits_left: usize, refs_left: usize },

    // cell_builder
    #[error("BuilderDataOverflow: Can't write {req} bits: only {left} free bits available")]
    BuilderDataOverflow { req: usize, left: usize },
    #[error("BuilderRefsOverflow: Can't write ref - 4 refs are written already")]
    BuilderRefsOverflow,
    #[error("BuilderNotEnoughData: Can't extract {required_bits} bits from {given} bytes")]
    BuilderNotEnoughData { required_bits: usize, given: usize },
    #[error("BuilderNumberBitsMismatch: Can't write number {number} as {bits} bits")]
    BuilderNumberBitsMismatch { number: String, bits: usize },
    #[error("BuilderMeta: Cell validation error: {0}")]
    BuilderMeta(String),

    // boc
    #[error("BOCEmpty: can't parse BOC from empty slice")]
    BOCEmpty,
    #[error("BOCWrongCellTypeTag: {0}")]
    BOCWrongCellTypeTag(u8),
    #[error("BOCSingleRoot: Expected 1 root, got {0}")]
    BOCSingleRoot(usize),
    #[error("BOCWrongMagic: {0}")]
    BOCWrongMagic(u32),
    #[error("BOCCustom: {0}")]
    BOCCustom(String),

    // tlb
    #[error("TLBWrongPrefix: Expecting prefix: {exp}, got: {given}, exp_bits={bits_exp}, left_bits={bits_left}")]
    TLBWrongPrefix {
        exp: usize,
        given: usize,
        bits_exp: usize,
        bits_left: usize,
    },
    #[error("TLBEnumOutOfOptions: Out of options")]
    TLBEnumOutOfOptions, // TODO collect errors from all options
    #[error("TLBObjectNoValue: No internal value found (method: {0})")]
    TLBObjectNoValue(String),
    #[error("TLBSnakeFormatUnsupportedBitsLen: Unsupported bits_len ({0})")]
    TLBSnakeFormatUnsupportedBitsLen(u32),
    #[error("TLBDictWrongKeyLen: Wrong key_bits_len: exp={exp}, got={got} for key={key}")]
    TLBDictWrongKeyLen { exp: usize, got: usize, key: BigUint },
    #[error("TLBDictEmpty: empty dict can't be written")]
    TLBDictEmpty,
    #[error("TLBWrongData: {0}")]
    TLBWrongData(String),

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
    #[error("MnemonicUnexpectedWordCount: expected 24 words, got {0}")]
    MnemonicUnexpectedWordCount(usize),
    #[error("MnemonicInvalidWord: {0}")]
    MnemonicInvalidWord(String),
    #[error("MnemonicInvalidFirstByte: {0}")]
    MnemonicInvalidFirstByte(u8),
    #[error("MnemonicInvalidPasslessFirstByte: {0}")]
    MnemonicInvalidPasslessFirstByte(u8),
    #[error("MnemonicPassHashError: {0}")]
    MnemonicPassHashError(String),

    // General errors
    #[error("UnexpectedValue: expected: {expected}, actual: {actual}")]
    UnexpectedValue { expected: String, actual: String },

    // TonActiveContract
    #[error("TonContractNotActive: contract {address} is not active at tx_id {tx_id:?}")]
    TonContractNotActive { address: TonAddress, tx_id: Option<TxId> },
    #[error("CustomError: {0}")]
    CustomError(String),

    // handling external errors
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    FromHex(#[from] FromHexError),
    #[error("{0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    NulError(#[from] std::ffi::NulError),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("{0}")]
    ElapsedError(#[from] tokio::time::error::Elapsed),
    #[error("{0}")]
    AdnlError(#[from] adnl::AdnlError),
    #[error("{0}")]
    ParseBigIntError(#[from] num_bigint::ParseBigIntError),
    #[error("{0}")]
    RecvError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("{0}")]
    VarError(#[from] VarError),
    #[error("{0}")]
    HmacInvalidLen(#[from] crypto_common::InvalidLength),
    #[error("{0:?}")]
    SignerError(nacl::Error),
    #[error("{0}")]
    BoxedError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    ArcTonlibError(#[from] Arc<TonlibError>),
}
