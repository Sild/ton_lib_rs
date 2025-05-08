use base64_serde::base64_serde_type;
base64_serde_type!(pub Base64Standard, base64::engine::general_purpose::STANDARD);

pub mod tl_req_ctx;
pub mod tl_request;
pub mod tl_response;
pub mod tl_types;
