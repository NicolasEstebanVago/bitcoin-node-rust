use std::io::Error;

use super::{block_header::BlockHeader, transaction::Transaction};
use crate::{
    helpers::auxiliar_functions::u8_to_hex_string,
    merkle_tree::merkle_tree_calculator::{calculate_merkle_tree, MerkleTreeError},
};
use bitcoin_hashes::{sha256d, Hash};
const GENESIS_BLOCK_HASH: &str = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";

#[derive(Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub txn_count: usize,
    pub txns: Vec<Transaction>,
}

/// Implementation of the blockheader structure
impl Block {
    /// creates the Blockheader structure with the parameters and returns itself
    pub fn new(header: BlockHeader, txn_count: usize, txns: Vec<Transaction>) -> Self {
        Block {
            header,
            txn_count,
            txns,
        }
    }

    fn is_genesis_block(&self) -> bool {
        let hash_string = u8_to_hex_string(&self.txns[0].hash);
        hash_string == GENESIS_BLOCK_HASH
    }

    pub fn is_valid(&self) -> Result<bool, MerkleTreeError> {
        // Validate the block header
        if !self.header.is_valid() {
            return Ok(false);
        }

        if self.txn_count == 0 {
            return Ok(true);
        };

        if self.is_genesis_block() {
            // Special case for genesis block
            Ok(self.txns[0].hash.to_vec() == self.header.merkle_root)
        } else {
            match self.merkle_root() {
                Ok(computed_root) => Ok(computed_root.to_vec() == self.header.merkle_root),
                Err(_) => Ok(false),
            }
        }
    }

    pub fn is_transaction_valid(&self, tx_hash: sha256d::Hash) -> bool {
        let proof_result = self.merkle_proof(tx_hash);

        if proof_result.is_err() {
            return false;
        }

        let merkle_root_result = self.merkle_root();

        if merkle_root_result.is_err() {
            return false;
        }

        // Si se pudo obtener la prueba de Merkle y la raíz del árbol de Merkle, verifica si son válidas
        let proof = proof_result.unwrap();
        let merkle_root = merkle_root_result.unwrap();
        self.verify_merkle_proof(proof, tx_hash, merkle_root)
    }

    fn verify_merkle_proof(
        &self,
        proof: Vec<(sha256d::Hash, bool)>,
        target: sha256d::Hash,
        merkle_root: sha256d::Hash,
    ) -> bool {
        let mut hash = target;
        for (node, is_right) in proof {
            hash = if is_right {
                self.compute_node(node, hash)
            } else {
                self.compute_node(hash, node)
            };
        }
        hash == merkle_root
    }

    pub fn merkle_root(&self) -> Result<sha256d::Hash, MerkleTreeError> {
        let tx_hashes = self.txns.iter().map(|tx| tx.hash).collect::<Vec<_>>();
        let merkle_tree_result = calculate_merkle_tree(tx_hashes)?;

        Ok(merkle_tree_result)
    }

    pub fn merkle_proof(
        &self,
        tx_hash: sha256d::Hash,
    ) -> Result<Vec<(sha256d::Hash, bool)>, MerkleTreeError> {
        let mut proof = Vec::new();
        let mut tx_hashes = self.txns.iter().map(|tx| tx.hash).collect::<Vec<_>>();
        let target_hash = tx_hash;

        let mut target_index = tx_hashes
            .iter()
            .position(|&hash| hash == target_hash)
            .ok_or(MerkleTreeError::TransactionNotFound)?;

        while tx_hashes.len() > 1 {
            if target_index % 2 == 0 {
                if target_index < tx_hashes.len() - 1 {
                    proof.push((tx_hashes[target_index + 1], false));
                } else {
                    proof.push((tx_hashes[target_index], false));
                }
            } else if target_index % 2 == 1 {
                proof.push((tx_hashes[target_index - 1], true));
            }

            let mut new_tx_hashes = Vec::new();
            for i in 0..tx_hashes.len() / 2 {
                let (left, right) = (tx_hashes[2 * i], tx_hashes[2 * i + 1]);
                let new_hash = self.compute_node(left, right);
                new_tx_hashes.push(new_hash);
            }

            if tx_hashes.len() % 2 == 1 {
                let last_hash = tx_hashes[tx_hashes.len() - 1];
                new_tx_hashes.push(self.compute_node(last_hash, last_hash));
            }

            tx_hashes = new_tx_hashes;
            target_index /= 2;
        }

        Ok(proof)
    }

    // Function to compute a new hash from a pair of hashes.
    // This is a helper function used in the construction of the Merkle tree.
    pub fn compute_node(&self, left: sha256d::Hash, right: sha256d::Hash) -> sha256d::Hash {
        let concatenated = [left.into_inner().as_ref(), right.into_inner().as_ref()].concat();
        sha256d::Hash::hash(&sha256d::Hash::hash(&concatenated).into_inner())
    }
}

#[allow(dead_code)]
pub fn get_valid_blocks(blocks: Vec<Block>) -> Result<Vec<Block>, Error> {
    let mut validated_blocks: Vec<Block> = vec![];

    for block in blocks {
        match block.is_valid() {
            Ok(true) => {
                validated_blocks.push(block);
            }
            Ok(false) => {}
            Err(e) => println!("Error validating block: {:?}", e),
        }
    }

    Ok(validated_blocks)
}
