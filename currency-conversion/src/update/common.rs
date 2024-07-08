use serde::Deserialize;
// TODO : Lock mechanism to avoid simultaneous update
/// Exchange rates API error response
#[derive(Deserialize, Debug)]
pub struct ErrorResponseAPI {
    // success : bool,
    pub error: ErrorInfoAPI,
}

/// Exchange rates API error information
#[derive(Deserialize, Debug)]
pub struct ErrorInfoAPI {
    pub code: String,
    pub message: String,
}
