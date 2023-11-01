use bitcoin_hashes::{sha256d, Hash};

/// #ENUM MerkleTreeError
/// Represents an enum whit possible values of errors in the generation of merkle tree
#[derive(Debug)]
pub enum MerkleTreeError {
    TransactionNotFound,
}

///Function to calculate merkle tree of a hash transactions list
pub fn calculate_merkle_tree(
    mut hash_list: Vec<sha256d::Hash>,
) -> Result<sha256d::Hash, MerkleTreeError> {
    if hash_list.len() == 1 {
        return Ok(hash_list[0]);
    }

    let mut new_hash_list = Vec::new();
    while !hash_list.is_empty() {
        let left = hash_list.remove(0);
        let right = if hash_list.is_empty() {
            left
        } else {
            hash_list.remove(0)
        };

        let concatenated = [left.into_inner().as_ref(), right.into_inner().as_ref()].concat();
        let new_hash = sha256d::Hash::hash(&sha256d::Hash::hash(&concatenated).into_inner());
        new_hash_list.push(new_hash);
    }

    calculate_merkle_tree(new_hash_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    pub fn test_calculate_merkle_tree_with_only_one_hash_in_the_list() {
        let hash = sha256d::Hash::hash(b"abcdef");
        let hash_list: Vec<sha256d::Hash> = vec![hash.clone()];

        let result = calculate_merkle_tree(hash_list);
        assert!(result.is_ok());

        let merkle_tree_root = result.unwrap();

        assert_eq!(merkle_tree_root, hash);
    }

    #[test]
    pub fn test_calculate_merkle_tree_with_two_hash_in_hash_list() {
        let hash = sha256d::Hash::hash(b"abcdef");
        let hash_list: Vec<sha256d::Hash> = vec![hash.clone(), hash.clone()];

        let result = calculate_merkle_tree(hash_list);
        assert!(result.is_ok());

        let merkle_tree_root = result.unwrap();
        let expected_hash = sha256d::Hash::from_str(
            "49c87b8126c7fe211f65f6df1f65651d2825748c709edab2bfd4913ed5e0e1ee",
        )
        .unwrap();

        assert_eq!(merkle_tree_root, expected_hash);
    }

    #[test]
    pub fn test_calculate_merkle_tree_with_more_than_two_hash_in_hash_list() {
        let hash = sha256d::Hash::hash(b"abcdef");
        let hash_list: Vec<sha256d::Hash> = vec![hash.clone(), hash.clone(), hash];

        let result = calculate_merkle_tree(hash_list);
        assert!(result.is_ok());

        let merkle_tree_root = result.unwrap();
        let expected_hash = sha256d::Hash::from_str(
            "9f51bfe21feeb680aaf19161da852577494589deea3b85388a91f034e24df91c",
        )
        .unwrap();

        assert_eq!(merkle_tree_root, expected_hash);
    }
}
