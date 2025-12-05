use thiserror::Error;

#[derive(Debug, Error)]
pub enum BtcError {
    #[error("Invalid Transaction")]
    InvalidTransaction,
    #[error("Invalid Block")]
    InvalidBlock,
    #[error("Invalid BlockHeader")]
    InvalidBlockHeader,
    #[error("Invalid TransactionOutput")]
    InvalidTransactionOutput,
    #[error("Invalid TransactionInput")]
    InvalidTransactionInput,
    #[error("Invalid MerkeleRoot")]
    InvalidMerkleRoot,
    #[error("Invalid BlockChain")]
    InvalidBlockChain,
    #[error("Invalid Hash")]
    InvalidHash,
    #[error("Invalid PrivateKey")]
    InvalidPrivateKey,
    #[error("Invalid PublicKey")]
    InvalidPublicKey,
    #[error("Invalid Signature")]
    InvalidSignature,
}

pub type Result<T> = std::result::Result<T, BtcError>;
