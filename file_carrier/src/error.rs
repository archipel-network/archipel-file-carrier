use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileCarrierError {
    #[error("ud3tn Error")]
    Ud3tnError(#[from] ud3tn_aap::Error),
    #[error("io Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("{0} is not a file carrier")]
    NotAFileCarrier(PathBuf),
    #[error("You are the first to use {0} as a file carrier")]
    FirstUser(PathBuf)
}