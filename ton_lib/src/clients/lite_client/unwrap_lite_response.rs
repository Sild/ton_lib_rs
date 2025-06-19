#[macro_export]
macro_rules! unwrap_lite_response {
    ($result:expr, $variant:ident) => {
        match $result {
            Response::$variant(inner) => Ok(inner),
            Response::Error(err) => Err(TLError::LiteClientErrorResponse(err)),
            _ => Err(TLError::LiteClientWrongResponse(stringify!($variant).to_string(), format!("{:?}", $result))),
        }
    };
}
