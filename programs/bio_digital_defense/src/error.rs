use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Fewer than POI_QUORUM Lymph Node validators signed this transaction")]
    InsufficientQuorum,
    #[msg("Gene is suppressed (Epigenetic_Status = 1); commit_gene cannot reactivate it")]
    GeneSuppressed,
    #[msg("gene_seq exceeds GENE_SEQ_MAX_LEN")]
    GeneSequenceTooLong,
}
