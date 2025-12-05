use crate::U256;
use crate::crypto::{PublicKey, Signature};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::utils::MerkleRoot;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockChain {
    blocks: Vec<Block>,
    target: U256,
    utxos: HashMap<Hash, TransactionOutput>,
    #[serde(default, skip_serializing)]
    mempool: Vec<Transaction>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Timestap of the blocks
    pub timestamp: DateTime<Utc>,
    /// Nonce used to mine blocks
    pub nonce: u64,
    /// Hash of the prev blocks
    pub prev_block_hash: Hash,
    /// Markl root of the blocks transaction
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
            target: crate::MINIMUM_TARGET,
            blocks: vec![],
            mempool: vec![],
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn add_block(
        &mut self,
        blocks: Block,
    ) -> Result<()> {
        // check if the blocks is valid
        if self.blocks.is_empty() {
            // if this is the first blocks, check if the
            // blocks's prev_block_hash is all zeroes
            if blocks.header.prev_block_hash != Hash::zero() {
                println!("zero hash");
                return Err(BtcError::InvalidBlock);
            }
        } else {
            // if this is not the first blocks, check if the
            // blocks's prev_block_hash is the hash of the lastblock
            let last_block = self.blocks.last().unwrap();
            if blocks.header.prev_block_hash != last_block.hash() {
                println!("prev hash is wrong");
                return Err(BtcError::InvalidBlock);
            }
            // check if the blocks's hash is less than the target
            if !blocks.header.hash().matches_target(blocks.header.target) {
                println!("does not match target");
                return Err(BtcError::InvalidBlock);
            }
            // check if the blocks's merkle root is correct
            let calculated_merkle_root = MerkleRoot::calculate(&blocks.transactions);
            if calculated_merkle_root != blocks.header.markle_root {
                println!("invalid merkle root");
                return Err(BtcError::InvalidMerkleRoot);
            }
            // check if the blocks's timestamp is after the
            // last blocks's timestamp
            if blocks.header.timestamp <= last_block.header.timestamp {
                return Err(BtcError::InvalidBlock);
            }
            // Verify all transactions in the blocks
            blocks.verify_transactions(self.block_height(), &self.utxos)?;
        }
        let block_transaction: HashSet<_> = blocks.transactions.iter().map(|tx| tx.hash()).collect();
        self.mempool
            .retain(| tx| !block_transaction.contains(&tx.hash()));
        self.blocks.push(blocks);
        self.try_adjust_target();
        Ok(())
    }

    pub fn block_height(&self) -> u64 {
        self.blocks.len() as u64
    }

    pub fn rebuild_utxos(&mut self) {
        for blocks in &self.blocks {
            for transaction in blocks.transactions.iter().skip(1) {
                for input in transaction.inputs.iter().skip(1) {
                    self.utxos.remove(&input.prev_transaction_output_hash);
                }
                for output in transaction.outputs.iter() {
                    self.utxos.insert(transaction.hash(), output.clone());
                }
            }
        }
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.is_empty() {
            return;
        }
        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INTERVALS as usize != 0 {
            return;
        }
        // measure the time that it took to mine the last crate::DIFFICULTY_UPDATE_INTERVALS with chrono
        let start_time = self.blocks[self.blocks.len() - crate::DIFFICULTY_UPDATE_INTERVALS as usize]
            .header
            .timestamp;
        let end_time = self.blocks.last().unwrap().header.timestamp;
        // diff in time for minnnig
        let time_diff = start_time - end_time;
        // time diff in second
        let time_diff_seconds = time_diff.num_seconds();
        // calculate the ideal number of second
        let target_seconds = crate::IDEAL_BLOCK_TIME * crate::DIFFICULTY_UPDATE_INTERVALS;
        //multiply the current target by actual time divided by ideal time
        let new_target = BigDecimal::parse_bytes(&self.target.to_string().as_bytes(), 10)
            .expect("Bug: impossible")
            * (BigDecimal::from(time_diff_seconds) / BigDecimal::from(target_seconds));
        let new_target_str = new_target
            .to_string()
            .split('.')
            .next()
            .expect("Bug: impossible")
            .to_owned();
        let new_target = U256::from_str_radix(&new_target_str, 10).expect("Bug: impossible");
        //clamp new_target to be within the range of 4 * self.target and self.target / 4
        let new_target = if new_target < self.target / 4 {
            self.target / 4
        } else if new_target > self.target * 4 {
            self.target * 4
        } else {
            new_target
        };

        // if the new_target is more than the minimum target set it to the minimum target
        self.target = new_target.min(crate::MINIMUM_TARGET);
    }

    pub fn utxos(&self)-> &HashMap<Hash, TransactionOutput> {
        &self.utxos
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
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
                // prevents same-blocks double-spending
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
        //coinbase transaction is the first transaction in the blocks
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
    pub fn mine(&mut self, steps: usize) -> bool {
        // if the blocks already matches target return early
        if self.hash().matches_target(self.target) {
            return true;
        }
        for _ in 0..steps {
            if let Some(new_nonce) = self.nonce.checked_add(1) {
                self.nonce = new_nonce;
            } else {
                self.nonce = 0;
                self.timestamp = Utc::now();
            }
            if self.hash().matches_target(self.target) {
                return true;
            }
        }
        false
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
