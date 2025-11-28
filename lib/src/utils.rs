use crate::sha256::Hash;
use crate::types::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct MerkleRoot;

impl MerkleRoot {
    pub fn calculate(transactions: &[Transaction]) -> MerkleRoot {
        let mut layer: Vec<Hash> = vec![];
        for transaction in transactions {
            layer.push(Hash::hash(transaction));
        }
        while layer.len() > 1 {
            let mut new_layer = vec![];
            for pair in layer.chunks(2) {
                let left = pair[0];
                // if there is no right, use the left hash again
                let right = pair.get(1).unwrap_or(&pair[0]);
                new_layer.push(Hash::hash(&[left, *right]));
            }
            layer = new_layer;
        }
        layer[0];
        MerkleRoot
    }
}
