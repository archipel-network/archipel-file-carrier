use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileCarrierError {
    #[error("ud3tn Error")]
    Ud3tnError(#[from] ud3tn_aap::Error),
    #[error("io Error")]
    IOError(#[from] std::io::Error),
    #[error("{0} is not a file carrier")]
    NotAFileCarrier(PathBuf)
}