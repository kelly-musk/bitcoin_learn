use crate::U256;
use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;
use crate::utils::MerkleRoot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Timestap of the block
    pub timestamp: DateTime<Utc>,
    /// Nonce used to mine block
    pub nonce: u64,
    /// Hash of the prev block
    pub prev_block_hash: Hash,
    /// Markl root of the block transaction
    pub markle_root: MerkleRoot,
    /// Target
    pub target: U256,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub public_key: PublicKey,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain { blocks: vec![] }
    }
    pub fn add_blocks(&mut self, blocks: Block) {
        self.blocks.push(blocks);
    }
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            header,
            transactions,
        }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        markle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            markle_root,
            target,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        Transaction {
            inputs: inputs,
            outputs: outputs,
        }
    }
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl TransactionOutput {
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}
