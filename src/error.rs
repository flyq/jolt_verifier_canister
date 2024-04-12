use candid::{CandidType, Deserialize};
use ic_exports::ic_cdk::api::call::RejectionCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Deserialize, CandidType, Eq, PartialEq)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("the user has no permission to call this method")]
    NotAuthorized,

    #[error("stable struct not found: {0}")]
    StableError(String),

    #[error("canister call error: {0}")]
    CallError(String),
}

impl From<(RejectionCode, String)> for Error {
    fn from(e: (RejectionCode, String)) -> Self {
        Self::CallError(format!("reject code is {:?}, err msg is {}", e.0, e.1))
    }
}
