use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Fewer than POI_QUORUM Lymph Node validators signed this transaction")]
    InsufficientQuorum,
}
