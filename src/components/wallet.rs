use bitcoin_hashes::{ripemd160, sha256d, Hash};

use rand::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

use crate::connection::connection_protocol::{fetch_nodes_config, set_tcp_stream_vec};
use crate::helpers::auxiliar_functions::u8_to_hex_string;
use crate::helpers::auxiliar_functions::{
    address_from_script, address_to_script_pubkey, bytes_to_hex, find_spent_utxo, hex_to_bytes,
    hex_to_bytes_rev,
};
use crate::testnet_protocol::broadcasting::broadcast_transaction;

use super::utxo_set::UTXOSet;
use super::utxo_struct::Utxo;
use super::{
    block::Block,
    transaction::{Transaction, TransactionInput, TransactionOutput},
};

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub private_key: SecretKey,
    pub public_key: secp256k1::PublicKey,
    pub utxo_set: UTXOSet,
    pub balance: u64,
    pub transactions_history: Vec<Transaction>,
    pub recieved_transactions: Vec<Transaction>,
    pub sent_transactions: Vec<Transaction>,
    pub utxos_vueltos: Vec<Utxo>,
}

impl Wallet {
    #[allow(dead_code)]
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();

        let mut rng = rand::thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);
        let private_key = SecretKey::from_slice(&key_bytes).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &private_key);

        let mut address_raw = Vec::new();
        address_raw
            .extend_from_slice(&sha256d::Hash::hash(&public_key.serialize()[..]).into_inner());
        address_raw.extend_from_slice(&ripemd160::Hash::hash(&address_raw).into_inner());
        let address = bs58::encode(address_raw).into_string();

        Wallet {
            address,
            private_key,
            public_key,
            balance: 0,
            transactions_history: Vec::new(),
            utxo_set: UTXOSet { utxos: Vec::new() },
            recieved_transactions: Vec::new(),
            sent_transactions: Vec::new(),
            utxos_vueltos: Vec::new(),
        }
    }

    pub fn new_from_existing(
        private_key: &secp256k1::SecretKey,
        public_key: &secp256k1::PublicKey,
        address_string: &str,
    ) -> Wallet {
        let address = address_string.to_string();
        let utxos: Vec<Utxo> = Vec::new();

        Wallet {
            address,
            private_key: *private_key,
            public_key: *public_key,
            balance: 0,
            transactions_history: Vec::new(),
            utxo_set: UTXOSet { utxos },
            recieved_transactions: Vec::new(),
            sent_transactions: Vec::new(),
            utxos_vueltos: Vec::new(),
        }
    }

    pub fn remove_utxo(&mut self, utxo: Utxo) {
        self.utxo_set.remove_utxo(&utxo.txid, utxo.index);
        self.utxos_vueltos
            .retain(|u| !(u.txid == utxo.txid && u.index == utxo.index));
    }

    pub fn calculate_balance(&mut self) -> u64 {
        let mut balance: u64 = 0;
        for utxo in &self.utxo_set.utxos {
            balance += utxo.value;
        }
        self.balance = balance;
        balance
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn update_utxo_set(&mut self, new_utxo: Utxo, transaction: Transaction, vuelto: bool) {
        if vuelto {
            self.utxos_vueltos.push(new_utxo.clone());
        }
        self.utxo_set.add_utxo(new_utxo);
        self.transactions_history.push(transaction);
    }

    pub fn create_sighash(&self, transaction: &Transaction) -> [u8; 32] {
        let mut tx_clone = transaction.clone();
        let bytes = tx_clone.to_bytes();
        transaction.print_hex();
        for input in tx_clone.inputs.iter_mut() {
            input.script = vec![];
        }

        let hash1 = sha256d::Hash::hash(&bytes);
        hash1.into_inner()
    }

    pub fn sign_transaction(&mut self, transaction: &mut Transaction) {
        let secp = Secp256k1::signing_only();

        let sighashes: Vec<_> = transaction
            .inputs
            .iter()
            .map(|_| self.create_sighash(transaction))
            .collect();

        for (input, sighash) in transaction.inputs.iter_mut().zip(sighashes) {
            let message = secp256k1::Message::from_slice(&sighash[..]).unwrap();
            let sig = secp.sign_ecdsa(&message, &self.private_key);
            let mut sig_ser = sig.serialize_der().to_vec();

            let pubkey_ser = self.public_key.serialize();

            sig_ser.push(0x01);
            let mut script_sig = vec![];
            script_sig.push(sig_ser.len() as u8);
            script_sig.extend(sig_ser);
            script_sig.push(pubkey_ser.len() as u8);
            script_sig.extend(&pubkey_ser);

            input.script = script_sig.clone();
        }
        let tx_ser = transaction.to_bytes();
        transaction.txid = bitcoin_hashes::sha256d::Hash::hash(&tx_ser)
            .into_inner()
            .to_vec();
        transaction.hash = bitcoin_hashes::sha256d::Hash::hash(&tx_ser);

        let hex_string = transaction.to_hex();

        let start_index = hex_string.len() - 8;
        let string_slice = &hex_string[..start_index];
        println!("Signed transaction: {:?} \n ", string_slice);

        println!(
            "Signed transaction on SHA256d: {}",
            bitcoin_hashes::sha256d::Hash::hash(&hex_to_bytes(string_slice))
        );

        let string_hash =
            bitcoin_hashes::sha256d::Hash::hash(&hex_to_bytes(string_slice)).to_string();

        let bytes_hash: Vec<u8> = string_hash
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let byte_string = unsafe { std::str::from_utf8_unchecked(chunk) };
                u8::from_str_radix(byte_string, 16).unwrap()
            })
            .collect();

        let reversed_bytes: Vec<u8> = bytes_hash.into_iter().rev().collect();

        println!(
            "HASH SIMULANDO PARSEO ---> {}",
            u8_to_hex_string(&reversed_bytes)
        );

        let nodes = fetch_nodes_config().unwrap();
        let tcp_stream_vec = set_tcp_stream_vec(nodes[..].to_vec());

        println!("Antes de broadcastear...");

        broadcast_transaction(hex_to_bytes(string_slice), tcp_stream_vec.unwrap());

        self.transactions_history.push(transaction.clone());
    }

    pub fn create_transaction(&mut self, recipient: &str, amount: u64) {
        let mut utxos_to_spent = Vec::new();
        let mut total_to_spend = 0;

        println!("self utxo vueltos : {:?}", self.utxos_vueltos.clone());
        println!(
            "CATNIDAD self utxo vueltos : {:?}",
            self.utxos_vueltos.len()
        );

        for utxo in self.utxos_vueltos.clone() {
            if total_to_spend >= amount {
                break;
            }
            total_to_spend += utxo.value;
            utxos_to_spent.push(utxo);
        }

        if self.calculate_balance() >= amount {
            // Create new transaction inputs from the chosen UTXOs
            let mut new_inputs = Vec::new();

            for utxo in utxos_to_spent {
                let combined_bytes_vec = [
                    hex_to_bytes_rev(&utxo.txid),
                    utxo.index.to_le_bytes().to_vec(),
                ]
                .concat();

                let mut bytes_arr: [u8; 36] = [0; 36];
                bytes_arr.copy_from_slice(&combined_bytes_vec[0..36]);
                let new_input = TransactionInput {
                    previous_output: bytes_arr,
                    script: address_to_script_pubkey(&self.address),
                    sequence: 0xffffffff,
                };
                new_inputs.push(new_input);
            }

            let mut new_outputs = Vec::new();

            new_outputs.push(TransactionOutput {
                value: amount,
                script_pubkey: address_to_script_pubkey(recipient),
            });

            println!("TOTAL TO SPEND : {}", total_to_spend);
            println!("AMOUNT : {}", amount);

            let change = total_to_spend - amount;
            if change > 0 {
                new_outputs.push(TransactionOutput {
                    value: change - 300,
                    script_pubkey: address_to_script_pubkey(&self.address),
                });
            }

            let mut transaction = Transaction {
                version: 1,                                          // standard
                hash: bitcoin_hashes::sha256d::Hash::hash(&[0; 32]), // temporary hash
                tx_in_count: new_inputs.len() as u32,
                inputs: new_inputs,
                tx_out_count: new_outputs.len() as u32,
                outputs: new_outputs,
                lock_time: 0000000000_u32, // standard
                txid: vec![],              // to be filled later
            };

            self.sign_transaction(&mut transaction);
        }
    }

    pub fn get_balance(&mut self) -> u64 {
        self.calculate_balance()
    }

    #[allow(dead_code)]
    pub fn verify_transaction(&self, tx_hash: sha256d::Hash, block: Block) -> bool {
        block.is_transaction_valid(tx_hash)
    }

    #[allow(dead_code)]
    pub fn recieve_transaction(&mut self, transaction: &Transaction) {
        self.recieved_transactions.push(transaction.clone());
    }

    #[allow(dead_code)]
    pub fn get_transactions_history(&mut self) -> Vec<Transaction> {
        self.transactions_history.clone()
    }

    pub fn get_recieved_transactions(&mut self) -> Vec<Transaction> {
        self.recieved_transactions.clone()
    }

    #[allow(dead_code)]
    pub fn get_sent_transactions(&mut self) -> Vec<Transaction> {
        self.sent_transactions.clone()
    }

    #[allow(dead_code)]
    pub fn get_available_balance(&self) -> u64 {
        self.balance
    }

    pub fn get_pending_balance(&self) -> u64 {
        self.balance
    }

    #[allow(dead_code)]
    pub fn get_inmature_balance(&self) -> u64 {
        self.balance
    }

    pub fn get_utxos(&mut self) -> Vec<Utxo> {
        self.utxo_set.utxos.clone()
    }
}

pub fn update_wallet(wallet: &mut Wallet, block: Block) {
    for (_index_tx, tx) in block.txns.iter().enumerate() {
        let mut vuelto = false;

        for (_index, input) in tx.inputs.iter().enumerate() {
            if let Some(utxo_to_remove) = find_spent_utxo(input, &wallet.utxo_set.utxos) {
                wallet.remove_utxo(utxo_to_remove);
                vuelto = true;
            }
        }

        for (_index, ouput) in tx.outputs.iter().enumerate() {
            let string = address_from_script(&ouput.script_pubkey);

            if string.is_some() && string.clone().unwrap() == wallet.address {
                let new_utxo = Utxo {
                    txid: bytes_to_hex(&tx.txid),
                    index: 1,
                    value: ouput.value,
                    pubkey: wallet.get_public_key(),
                };

                let mut repetido = false;
                if wallet.get_utxos().is_empty() {
                    for utxo in &wallet.get_utxos() {
                        if utxo.txid == new_utxo.clone().txid {
                            repetido = true;
                        }
                    }

                    if !repetido {
                        wallet.update_utxo_set(new_utxo.clone(), tx.clone(), vuelto);
                    }
                } else {
                    wallet.update_utxo_set(new_utxo.clone(), tx.clone(), vuelto);
                }
            }
        }

        for (_index, input) in tx.inputs.iter().enumerate() {
            if let Some(utxo_to_remove) = find_spent_utxo(input, &wallet.utxo_set.utxos) {
                wallet.remove_utxo(utxo_to_remove);
            }
        }
    }
}
