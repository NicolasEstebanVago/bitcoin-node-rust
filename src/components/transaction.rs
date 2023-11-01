use crate::helpers::auxiliar_functions::{address_from_script, bytes_to_hex, find_spent_utxo};

use super::{utxo_struct::Utxo, wallet::Wallet};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: bitcoin_hashes::sha256d::Hash,
    pub version: u32,
    pub tx_in_count: u32,
    pub inputs: Vec<TransactionInput>,
    pub tx_out_count: u32,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u32,
    pub txid: Vec<u8>,
}

impl Transaction {
    pub fn to_hex(&self) -> String {
        self.to_bytes()
            .iter()
            .map(|byte| {
                let hex_value = format!("{:02x}", byte);
                hex_value
            })
            .collect()
    }

    fn to_hex_string(&self, bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|byte: &u8| format!("{:02x}", byte))
            .collect()
    }

    fn to_hex_string_le(&self, bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|byte: &u8| format!("{:02x}", byte))
            .collect()
    }
    pub fn get_amount(&self, wallet: &mut Wallet) -> u64 {
        let mut vec_outputs: Vec<Utxo> = Vec::new();
        for (_index, ouput) in self.outputs.iter().enumerate() {
            let string = address_from_script(&ouput.script_pubkey);

            if string.is_some() && string.clone().unwrap() == wallet.address {
                let new_utxo = Utxo {
                    txid: bytes_to_hex(&self.txid),
                    index: 1,
                    value: ouput.value,
                    pubkey: wallet.get_public_key(),
                };
                vec_outputs.push(new_utxo);
            }
        }

        for (_index, input) in self.inputs.iter().enumerate() {
            if let Some(utxo_to_remove) = find_spent_utxo(input, &wallet.utxo_set.utxos) {
                vec_outputs.retain(|x| x.txid != utxo_to_remove.txid);
            }
        }

        let mut value = 0;
        for i in vec_outputs {
            value += i.value
        }

        value
    }
    pub fn print_hex(&self) {
        println!(
            "Version: {}",
            self.to_hex_string_le(&self.version.to_le_bytes())
        );
        println!("Tx_in_count: {:02x}", self.tx_in_count);
        for (_i, input) in self.inputs.iter().enumerate() {
            println!(
                "Previous Output: {}",
                self.to_hex_string(&input.previous_output)
            );
            println!("len Script: {:02x}", (&input.script.len()));
            println!("Script: {}", self.to_hex_string(&input.script));
            println!(
                "Sequence: {}",
                self.to_hex_string_le(&input.sequence.to_le_bytes())
            );
        }
        println!("Tx_out_count: {:02x}", self.tx_out_count);
        for (_i, output) in self.outputs.iter().enumerate() {
            println!(
                "Value: {}",
                self.to_hex_string_le(&output.value.to_le_bytes())
            );
            println!(
                "Script PubKey: {}",
                self.to_hex_string(&output.script_pubkey)
            );
        }
        println!(
            "Lock_time: {}",
            self.to_hex_string_le(&self.lock_time.to_le_bytes())
        );
        println!(
            "hash code type: {}\n",
            self.to_hex_string_le(&[0x01, 0x00, 0x00, 0x00])
        );
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        // Bitcoin utiliza little-endian para el version y lock_time
        bytes.extend(&self.version.to_le_bytes());

        bytes.push(self.inputs.len() as u8);
        for input in &self.inputs {
            bytes.extend(input.to_bytes());
        }

        bytes.push(self.outputs.len() as u8);
        for output in &self.outputs {
            bytes.extend(output.to_bytes());
        }

        bytes.extend(&self.lock_time.to_le_bytes());
        bytes.extend(&[0x01, 0x00, 0x00, 0x00]);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TransactionInput {
    pub previous_output: [u8; 36],
    pub script: Vec<u8>,
    pub sequence: u32,
}

impl TransactionOutput {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        // Bitcoin utiliza little-endian para el valor
        bytes.extend(&self.value.to_le_bytes());
        bytes.push(self.script_pubkey.len() as u8);
        bytes.extend(&self.script_pubkey);

        bytes
    }
}

impl TransactionInput {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(&self.previous_output);
        bytes.push(self.script.len() as u8);
        bytes.extend(&self.script);
        bytes.extend(&self.sequence.to_le_bytes());

        bytes
    }
}
