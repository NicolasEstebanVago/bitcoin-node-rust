use std::{fs::File, io, str::FromStr};

use bitcoin_hashes::sha256d;

use secp256k1::Secp256k1;

use crate::helpers::{
    auxiliar_functions::hex_string_to_reversed_bytes_block_hash, persistance::get_blocks_from_file,
};

use super::{transaction::Transaction, wallet::Wallet};

pub struct User {
    pub name: String,
    pub wallets: Vec<Wallet>,
}

impl User {
    pub fn new(name: String) -> Self {
        User {
            name,
            wallets: Vec::new(),
        }
    }

    pub fn create_new_wallet(&mut self, private_key: &secp256k1::SecretKey, address_string: &str) {
        let wallet = Wallet::new_from_existing(
            private_key,
            &private_key.public_key(&Secp256k1::new()),
            address_string,
        );

        self.wallets.push(wallet);
    }

    pub fn get_wallets(&mut self) -> &mut Vec<Wallet> {
        &mut self.wallets
    }

    #[allow(dead_code)]
    pub fn get_balance(&mut self) -> u64 {
        self.get_wallets()[0].get_balance()
    }

    #[allow(dead_code)]
    pub fn get_pending_balance(&mut self) -> u64 {
        self.get_wallets()[0].get_pending_balance()
    }

    #[allow(dead_code)]
    pub fn create_transaction(&mut self, recipient: &str, amount: u64, address_string: &str) {
        for wallet in self.get_wallets() {
            if wallet.address == address_string {
                wallet.create_transaction(recipient, amount);
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_all_transactions(&mut self, address_string: &str) -> Vec<Transaction> {
        for wallet in self.get_wallets() {
            if wallet.address == address_string {
                return wallet.get_transactions_history();
            }
        }

        Vec::new()
    }

    #[allow(dead_code)]
    pub fn get_recieved_transactions(&mut self, address_string: &str) -> Vec<Transaction> {
        for wallet in self.get_wallets() {
            if wallet.address == address_string {
                return wallet.get_recieved_transactions();
            }
        }

        Vec::new()
    }

    #[allow(dead_code)]
    pub fn get_sent_transactions(&mut self, address_string: &str) -> Vec<Transaction> {
        for wallet in self.get_wallets() {
            if wallet.address == address_string {
                return wallet.get_transactions_history();
            }
        }

        Vec::new()
    }
}

pub fn is_tx_valid_in_block(tx_hash_string: String, block_hash: String) -> bool {
    let block_hash_bytes = hex_string_to_reversed_bytes_block_hash(&block_hash).unwrap();
    let tx_hash = sha256d::Hash::from_str(&tx_hash_string).unwrap();

    let reader = io::BufReader::new(File::open("logs/blocks_proof_of_inclusion_test.txt").unwrap());
    let lista_blocks = get_blocks_from_file(reader);

    for (index, block) in lista_blocks.iter().enumerate() {
        if block.header.prev_block_hash == block_hash_bytes && index != 0 {
            if lista_blocks[index - 1].header.prev_block_hash != block_hash_bytes {
                return lista_blocks[index - 1].is_transaction_valid(tx_hash);
            } else {
                return lista_blocks[index - 2].is_transaction_valid(tx_hash);
            }
        }
    }

    false
}

// Tests depend on the current blocks file
/*
#[cfg(test)]
mod tests {
    use secp256k1::SecretKey;

    use crate::{
        components::{block::Block, block_header::BlockHeader},
        update_wallet,
    };

    use super::*;

    struct TestSetup {
        user: User,
        lista_blocks: Vec<Block>,
    }

    impl TestSetup {
        fn new() -> Self {
            let private_key = SecretKey::from_str(
                "B0A54F06D71E3739533DE0C7BAA535CD4EEEC7A0653E04AC6C2DDAD190A2EFA9",
            )
            .unwrap();

            let mut user = User::new("Nico".to_owned());
            user.create_new_wallet(&private_key, "mypPe9yK6S5GFEtU4Jd74F7wyh91x5bbkc");

            let reader = io::BufReader::new(File::open("logs/blocks.txt").unwrap());
            let lista_blocks = get_blocks_from_memory(reader);

            for mut wallet in user.get_wallets() {
                for validated_block in &lista_blocks {
                    update_wallet(&mut wallet, validated_block.clone());
                }
            }

            TestSetup { user, lista_blocks }
        }
    }

    fn test_txs(setup: &TestSetup, transactions: &[(&str, &str)], expected: bool) {
        for (hash_tx, hash_block) in transactions {
            assert_eq!(
                setup
                    is_tx_valid_in_block(hash_tx.to_string(), hash_block.to_string()),
                expected
            );
        }
    }

    #[test]
    pub fn test_validate_tx_are_valid_1() {
        let setup = TestSetup::new();
        let transactions = [(
            "e0e7d889e0d22ae1befab1ad15170184da265a62bd0769e229f9798458a63a43",
            "00000000000057ce73dd26b284a9665142cb55c3c8ddb2793d426a07d52d31fb",
        )];
        test_txs(&setup, &transactions, true);
    }

    #[test]
    pub fn test_validate_tx_are_not_valid_1() {
        let setup = TestSetup::new();
        let transactions = [
            (
                "fbe1b11cb3d97e54b2780da9614ab9d366fcb7ba5c8d081a6be28252439f98d6",
                "0000000000000003917093170db721863a8a052cb705061682c31e2dc55d3663",
            ),
            (
                "d02c17a80f70685c3395a49090063656609746c3fbf85f1d6229de6b7ec54b41",
                "0000000000000003917093170db721863a8a052cb705061682c31e2dc55d3663",
            ),
            (
                "198d9522684bb9427ea8f79d73531c5b700ec10acb9c09338e0e11edf0b06072",
                "0000000000000003917093170db721863a8a052cb705061682c31e2dc55d3663",
            ),
        ];
        test_txs(&setup, &transactions, false);
    }

    #[test]
    pub fn test_validate_tx_are_valid_2() {
        let setup = TestSetup::new();
        let transactions = [
            (
                "ca2d47bf741b100439d5afb0eade38f5a01b744aeb86ca6505207056f37f1588",
                "00000000000025112ba58041b3b5edbd352aeab212d51f4376262694393422f0",
            ),
            (
                "d0f181b2b5d00ab604a2255d5094493825a668ef73652de8c5ce759d76b83985",
                "00000000000025112ba58041b3b5edbd352aeab212d51f4376262694393422f0",
            ),
            (
                "21497a38a6b27e6c2e6c03fe91afcae5b26f12deb28c5837b4e1fc72074b4f70",
                "00000000000025112ba58041b3b5edbd352aeab212d51f4376262694393422f0",
            ),
        ];
        test_txs(&setup, &transactions, true);
    }

    #[test]
    pub fn test_validate_tx_are_not_valid_2() {
        let setup = TestSetup::new();
        let transactions = [
            (
                "fbe1b11cb3d97e54b2780da9614ab9d366fcb7ba5c8d081a6be28252439f98d6",
                "00000000000025112ba58041b3b5edbd352aeab212d51f4376262694393422f0",
            ),
            (
                "d02c17a80f70685c3395a49090063656609746c3fbf85f1d6229de6b7ec54b41",
                "00000000000025112ba58041b3b5edbd352aeab212d51f4376262694393422f0",
            ),
            (
                "d02c17a80f70685c3395a49090063656609746c3fbf85f1d6229de6b7ec54b40",
                "00000000000025112ba58041b3b5edbd352aeab212d51f4376262694393422f0",
            ),
        ];
        test_txs(&setup, &transactions, false);
    }
}
*/
