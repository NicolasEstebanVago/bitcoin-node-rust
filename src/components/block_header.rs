extern crate bitcoin_hashes;
extern crate chrono;
extern crate rand;

use bitcoin_hashes::{sha256d, Hash};
use chrono::Utc;

const MAX_BITS: u32 = 0x1d00ffff;
const ZERO_HASH: [u8; 32] = [0; 32];

/// Represents the structure of the block header
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub version: u32,             /* represents the version*/
    pub prev_block_hash: Vec<u8>, /*represents the hash of the previous block*/
    pub merkle_root: Vec<u8>, /*represents the hash of the tree root of the merkle of the transactions included in the block*/
    pub timestamp: u32,       /*represents the time of the block*/
    pub bits: u32,            /* represents the difficulty of the current goal of the blockchain*/
    pub nonce: u32,           /* random number used for minery*/
}

/// Implementation of the blockheader structure
impl BlockHeader {
    /// creates the Blockheader structure with the parameters and returns itself
    pub fn new(
        version: u32,
        prev_block_hash: Vec<u8>,
        merkle_root: Vec<u8>,
        timestamp: u32,
        bits: u32,
        nonce: u32,
    ) -> Self {
        BlockHeader {
            version,
            prev_block_hash,
            merkle_root,
            timestamp,
            bits,
            nonce,
        }
    }

    fn calculate_hash(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&self.prev_block_hash);
        data.extend_from_slice(&self.merkle_root);
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.bits.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());

        let hash = sha256d::Hash::hash(&data);
        hash.into_inner().to_vec()
    }

    fn validate_pow(&self) -> bool {
        // Compactar los bits
        let target = target_from_compact(self.bits);
        // hash in little endian
        let hash = self.calculate_hash();
        // Big endian comparison gives error if the target is bigger than the hash
        hash.iter().rev().cmp(target.iter()) != std::cmp::Ordering::Greater
    }

    /// validates the property version
    fn validate_version(&self) -> bool {
        true
    }

    fn validate_prev_blockhash(&self) -> bool {
        ZERO_HASH != self.prev_block_hash.as_slice() && self.prev_block_hash.len() == 32
    }

    fn validate_merkle_root(&self) -> bool {
        ZERO_HASH != self.merkle_root.as_slice() && self.merkle_root.len() == 32
    }

    fn validate_time(&self) -> bool {
        let current_time = Utc::now().timestamp() as u32;
        self.timestamp <= current_time
    }

    fn validate_bits(&self) -> bool {
        self.bits <= MAX_BITS
    }

    fn validate_nonce(&self) -> bool {
        true
    }

    pub fn is_valid(&self) -> bool {
        //Validacion estructura general de los campos
        if !(self.validate_version()
            && self.validate_prev_blockhash()
            && self.validate_merkle_root()
            && self.validate_time()
            && self.validate_bits()
            && self.validate_nonce()
            && self.validate_pow())
        {
            return false;
        }

        true
    }
}

// Compact the bits for the comparison with the hash
fn target_from_compact(bits: u32) -> Vec<u8> {
    let size = bits >> 24;
    let word = bits & 0xFFFFFF;
    let mut target = vec![];

    if size <= 3 {
        target.resize((3 - size) as usize, 0);
        target.extend_from_slice(&word.to_le_bytes()[0..size as usize]);
    } else {
        target.extend_from_slice(&word.to_le_bytes());
        target.resize(size as usize, 0);
    }

    target
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_validate_blockheader_with_all_parameters_valid() {
        let block_header = BlockHeader::new(
            70015,
            [
                118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224, 226, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            [
                7, 112, 105, 110, 103, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 240, 189, 19, 49, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            245608,
            4295810,
            123456,
        );
        assert!(block_header.is_valid());
    }

    #[test]
    pub fn test_validate_blockheader_with_invalid_version() {
        let block_header = BlockHeader::new(
            70016,
            [
                118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224, 226, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            [
                7, 112, 105, 110, 103, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 240, 189, 19, 49, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            245608,
            4295810,
            123456,
        );
        assert!(!block_header.is_valid());
        assert!(!block_header.validate_version());
    }

    #[test]
    pub fn test_validate_blockheader_with_invalid_prev_block_hash() {
        let block_header = BlockHeader::new(
            70015,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
            .to_vec(),
            [
                7, 112, 105, 110, 103, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 240, 189, 19, 49, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            245608,
            4295810,
            123456,
        );

        assert!(!block_header.is_valid());
        assert!(!block_header.validate_prev_blockhash());
    }
    #[test]
    pub fn test_validate_blockheader_with_invalid_merkle_root() {
        let block_header = BlockHeader::new(
            70015,
            [
                118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224, 226, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
            .to_vec(),
            245608,
            4295810,
            123456,
        );

        assert!(!block_header.is_valid());
        assert!(!block_header.validate_merkle_root());
    }
    #[test]
    pub fn test_validate_blockheader_with_invalid_timestamp() {
        let block_header = BlockHeader::new(
            70015,
            [
                118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224, 226, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            [
                7, 112, 105, 110, 103, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 240, 189, 19, 49, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            1685923891,
            4295810,
            123456,
        );
        assert!(!block_header.is_valid());
        assert!(!block_header.validate_time());
    }

    #[test]
    pub fn test_validate_blockheader_with_invalid_bits() {
        let block_header = BlockHeader::new(
            70015,
            [
                118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224, 226, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            [
                7, 112, 105, 110, 103, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 240, 189, 19, 49, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            245608,
            486604812,
            123456,
        );
        assert!(!block_header.is_valid());
        assert!(!block_header.validate_bits());
    }

    #[test]
    pub fn test_validate_blockheader_with_invalid_nonce() {
        let block_header = BlockHeader::new(
            70015,
            [
                118, 101, 114, 97, 99, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 93, 246, 224, 226, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            [
                7, 112, 105, 110, 103, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 240, 189, 19, 49, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .to_vec(),
            245608,
            4295810,
            4294967295,
        );
        assert!(!block_header.is_valid());
        assert!(!block_header.validate_nonce());
    }
}
