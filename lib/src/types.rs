use crate::U256;
use crate::crypto::{PublicKey, Signature};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::utils::MerkleRoot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockChain {
    pub block: Vec<Block>,
    pub utxos: HashMap<Hash, TransactionOutput>,
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

impl BlockChain {
    pub fn new() -> Self {
        BlockChain {
            utxos: HashMap::new(),
            block: vec![],
        }
    }
    pub fn add_blocks(&mut self, block: Block) -> Result<()> {
        // check if block is valid
        if self.block.is_empty() {
            // if this is my first block check if the block's
            // prev's_block_hash is all zeros
            if block.header.prev_block_hash != Hash::zero() {
                println!("Zero Hash");
                return Err(BtcError::InvalidBlock);
            } else {
                // if this is not the first bloclk check the
                // block's prev_block_hash is the last hash
                let last_block = self.block.last().unwrap();
                if block.header.prev_block_hash != last_block.hash() {
                    println!("prev hash is wrong ");
                    return Err(BtcError::InvalidBlock);
                }
                // check if block hash is lesser than the target
                if !block.header.hash().matches_target(block.header.target) {
                    println!("dose not match target");
                    return Err(BtcError::InvalidBlock);
                }
                //check if the block merkle root is correct
                let calculated_merkle_root = MerkleRoot::calculate(&block.transactions);
                if calculated_merkle_root != block.header.markle_root {
                    println!("Invalid merkle root");
                    return Err(BtcError::InvalidMerkeleRoot);
                }
                // check if the timestamp is after the last blck timestamp
                if block.header.timestamp != last_block.header.timestamp {
                    println!("Invalid time stamp");
                    return Err(BtcError::InvalidBlock);
                }
                // verify all transaction in a block
                // return block.verify_transactions(&self.block_height(), &self.utxos);
                // return block.verify_transactions( &self.utxos);
            }
        }
        self.block.push(block);
        Ok(())
    }

    pub fn rebuild_utxos(&mut self) {
        for block in &self.block {
            for transaction in block.transactions.iter().skip(1) {
                for input in transaction.inputs.iter().skip(1) {
                    self.utxos.remove(&input.prev_transaction_output_hash);
                    {
                        for output in transaction.outputs.iter() {
                            self.utxos.insert(transaction.hash(), output.clone());
                        }
                    }
                }
            }
        }
    }
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            header,
            transactions,
        }
    }

    pub fn calculate_miner_fees(&self, utxos: &HashMap<Hash, TransactionOutput>) -> Result<u64> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        // check transsctions after coinbase
        for transaction in self.transactions.iter().skip(1) {
            for input in &transaction.inputs {
                // for input in transaction.inputs.iter().skip(1) //a more prefferd way by me
                // input do not contain the values of outputs so we need to match inputs to outputs
                let prev_outputs = utxos.get(&input.prev_transaction_output_hash);
                if prev_outputs.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_outputs.unwrap();
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }
                inputs.insert(input.prev_transaction_output_hash, prev_output.clone());
            }
            for output in transaction.outputs.iter().skip(1) {
                if outputs.contains_key(&output.hash()) {
                    return Err(BtcError::InvalidTransaction);
                }
                outputs.insert(output.hash(), output.clone());
            }
        }
        let input_value: u64 = inputs.values().map(|output| output.value).sum();
        let output_value: u64 = outputs.values().map(|output| output.value).sum();
        Ok(input_value - output_value)
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn verify_transactions(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<()> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        // reject completely empty blocks
        if self.transactions.is_empty() {
            println!("Empty transaction");
            return Err(BtcError::InvalidTransaction);
        }
        // verify coinbase transaction
        self.verify_coinbase_transaction(predicted_block_height, utxos)?;
        for transaction in self.transactions.iter().skip(1) {
            let mut input_value = 0;
            let mut otput_value = 0;
            for input in transaction.inputs.iter().skip(1) {
                let prev_outputs = utxos.get(&input.prev_transaction_output_hash);
                if prev_outputs.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_outputs.unwrap();
                // prevents same-block double-spending
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }
                // check if the signature is valid
                if !input
                    .signature
                    .verify(&input.prev_transaction_output_hash, &prev_output.public_key)
                {
                    return Err(BtcError::InvalidSignature);
                }
                input_value += prev_output.value;
                inputs.insert(input.prev_transaction_output_hash, prev_output.clone());
            }
            for output in transaction.outputs.iter().skip(1) {
                otput_value += output.value;
            }
            // its fine for output value to be less than input value
            // as difference is fee for miners
            if input_value < otput_value {
                return Err(BtcError::InvalidTransaction);
            }
        }
        Ok(())
    }

    pub fn verify_coinbase_transaction(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<()> {
        //coinbase transaction is the first transaction in the block
        let coinbase_transaction = &self.transactions[0];
        if coinbase_transaction.inputs.len() != 0 {
            return Err(BtcError::InvalidTransaction);
        }
        if coinbase_transaction.outputs.len() == 0 {
            return Err(BtcError::InvalidTransaction);
        }
        let miner_fees = self.calculate_miner_fees(utxos)?;
        let block_reward = crate::INITIAL_REWARD * 10u64.pow(8)
            / 2u64.pow((predicted_block_height / crate::HALVING_INTERVAL) as u32);
        let total_coinbase_outputs: u64 = coinbase_transaction
            .outputs
            .iter()
            .map(|output| output.value)
            .sum();
        if total_coinbase_outputs != block_reward + miner_fees {
            return Err(BtcError::InvalidTransaction);
        }
        Ok(())
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
