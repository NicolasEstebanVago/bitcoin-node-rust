use std::{
    fs::File,
    io::{self, BufRead},
};

use bitcoin_hashes::Hash;

use crate::{
    components::{
        block::Block,
        block_header::BlockHeader,
        transaction::{Transaction, TransactionInput, TransactionOutput},
    },
    helpers::auxiliar_functions::{split_ignore_brackets, str_to_bytes},
};

use super::auxiliar_functions::{
    get_value_after_keyword, hex_to_bytes_rev, parse_inputs, parse_output_values, reverse_hash,
    split_ignore_curly_brackets, split_transaction_inputs, split_transaction_outputs,
};

pub fn get_transaction_from_file(result: Vec<String>) -> Transaction {
    let hash = get_value_after_keyword(&result[0], "hash : ").unwrap();
    let version = get_value_after_keyword(&result[1], "version : ").unwrap();
    let tx_in_count = get_value_after_keyword(&result[2], "tx_in_count : ").unwrap();
    let tx_out_count = get_value_after_keyword(&result[4], "tx_out_count : ").unwrap();
    let lock_time = get_value_after_keyword(&result[6], "lock_time : ").unwrap();
    let txid = get_value_after_keyword(&result[7], "txid : ").unwrap();

    let outputs = split_transaction_outputs(&result[5]);
    let parsed_outputs = parse_output_values(outputs);
    let mut vec_outputs: Vec<TransactionOutput> = Vec::new();
    for output in parsed_outputs {
        let transaction_output = TransactionOutput {
            value: output.0,
            script_pubkey: output.1,
        };

        vec_outputs.push(transaction_output);
    }

    let inputs = split_transaction_inputs(&result[3]);
    let parsed_inputs = parse_inputs(inputs);
    let mut vec_inputs: Vec<TransactionInput> = Vec::new();
    for input in parsed_inputs {
        let tx_input = TransactionInput {
            previous_output: input.0.try_into().unwrap(),
            script: input.1,
            sequence: input.2,
        };

        vec_inputs.push(tx_input);
    }
    let txid: Vec<u8> = txid
        .split(", ") // Split the string by comma and space
        .map(|num_str| num_str.parse().unwrap()) // Parse each number string to u8
        .collect(); //

    Transaction {
        hash: bitcoin_hashes::sha256d::Hash::from_slice(&hex_to_bytes_rev(&hash)).unwrap(),
        version: version.parse().unwrap(),
        tx_in_count: tx_in_count.parse::<u32>().unwrap(),
        inputs: vec_inputs,
        tx_out_count: tx_out_count.parse::<u32>().unwrap(),
        outputs: vec_outputs,
        lock_time: lock_time.parse::<u32>().unwrap(),
        txid,
    }
}

pub fn get_blocks_from_file(reader: io::BufReader<File>) -> Vec<Block> {
    let mut vec_blocks = Vec::new();
    for line in reader.lines() {
        let list: Vec<String> = line
            .unwrap()
            .split('|')
            .map(|s| s.trim().to_string())
            .collect();
        let list = list.clone()[1..].to_vec();

        for item in list {
            if !item.is_empty() {
                let start1 = item.find('(').unwrap() + 1;
                let end1 = item[start1..].find(')').unwrap() + start1;
                let header1 = item[start1..end1].to_string();

                let items = split_ignore_brackets(&header1);

                let block_header = BlockHeader {
                    version: items[0].parse().unwrap(),
                    prev_block_hash: str_to_bytes(&items[1]),
                    merkle_root: str_to_bytes(&items[2]),
                    timestamp: items[3].parse().unwrap(),
                    bits: items[4].parse().unwrap(),
                    nonce: items[5].parse().unwrap(),
                };

                let start2 = item[end1..].find('(').unwrap() + 1 + end1;
                let end2 = item[start2..].find(')').unwrap() + start2;
                let txs = item[start2..end2].to_string();
                let parts: Vec<&str> = txs.split(" hash : ").collect();
                let mut result: Vec<String> = Vec::new();

                for (i, part) in parts.iter().enumerate() {
                    if i != 0 || part.trim() != "" {
                        let mut item = "hash : ".to_owned();
                        item.push_str(part);
                        result.push(item);
                    }
                }

                result = result[1..].to_vec();
                let mut vec_txs: Vec<Transaction> = Vec::new();

                for tx in result {
                    let result = split_ignore_curly_brackets(&tx);
                    let transaction = get_transaction_from_file(result);
                    vec_txs.push(transaction);
                }

                let block = Block {
                    header: block_header.clone(),
                    txn_count: vec_txs.len(),
                    txns: vec_txs,
                };

                vec_blocks.push(block);
            }
        }
    }
    vec_blocks
}

pub fn get_headers_from_memory(
    headers_list: &Vec<BlockHeader>,
    hash_start: &[u8],
    hash_stop: &[u8],
) -> Vec<BlockHeader> {
    let mut start_found = false;
    let mut vec_headers = Vec::new();

    for header in headers_list {
        if header.prev_block_hash == hash_start {
            start_found = true;
        }

        if start_found {
            vec_headers.push(header.clone());
        }

        if header.prev_block_hash == hash_stop || vec_headers.len() == 2000 {
            break;
        }
    }
    vec_headers
}

pub fn get_headers_from_file(reader: io::BufReader<File>) -> Vec<BlockHeader> {
    let mut vec_headers: Vec<BlockHeader> = Vec::new();

    for line in reader.lines() {
        let list: Vec<String> = line
            .unwrap()
            .split('|')
            .map(|s| s.trim().to_string())
            .collect();

        let list = list.clone()[1..].to_vec();

        for item in list {
            if !item.is_empty() {
                let start1 = item.find('(').unwrap() + 1;
                let end1 = item[start1..].find(')').unwrap() + start1;
                let header1 = item[start1..end1].to_string();

                let items = split_ignore_brackets(&header1);

                let block_header = BlockHeader {
                    version: items[0].parse().unwrap(),
                    prev_block_hash: str_to_bytes(&items[1]),
                    merkle_root: str_to_bytes(&items[2]),
                    timestamp: items[3].parse().unwrap(),
                    bits: items[4].parse().unwrap(),
                    nonce: items[5].parse().unwrap(),
                };

                vec_headers.push(block_header);
            }
        }
    }

    vec_headers
}

#[allow(dead_code)]
pub fn get_tx_from_hash(
    reader: io::BufReader<File>,
    tx_hash: Option<&[u8]>,
) -> Option<Transaction> {
    for line in reader.lines() {
        let list: Vec<String> = line
            .unwrap()
            .split('|')
            .map(|s| s.trim().to_string())
            .collect();
        let list = list.clone()[1..].to_vec();

        for item in list {
            if !item.is_empty() {
                let start1 = item.find('(').unwrap() + 1;
                let end1 = item[start1..].find(')').unwrap() + start1;

                let start2 = item[end1..].find('(').unwrap() + 1 + end1;
                let end2 = item[start2..].find(')').unwrap() + start2;
                let txs = item[start2..end2].to_string();
                let parts: Vec<&str> = txs.split(" hash : ").collect();
                let mut result: Vec<String> = Vec::new();

                for (i, part) in parts.iter().enumerate() {
                    if i != 0 || part.trim() != "" {
                        let mut item = "hash : ".to_owned();
                        item.push_str(part);
                        result.push(item);
                    }
                }

                result = result[1..].to_vec();

                for tx in result {
                    let result = split_ignore_curly_brackets(&tx);
                    let transaction = get_transaction_from_file(result);
                    if let Some(block_hash) = tx_hash {
                        if reverse_hash(block_hash) == transaction.hash.to_vec() {
                            return Some(transaction);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn get_blocks_from_memory(blocks: &Vec<Block>, hash: &[u8]) -> Option<Block> {
    let mut vec_total: Vec<Block> = Vec::new();

    for block in blocks {
        if block.header.prev_block_hash == hash {
            let last_block = vec_total.last();
            return last_block.cloned();
        }

        vec_total.push(block.clone());
    }

    None
}

pub fn get_tx_from_memory(blocks: &Vec<Block>, hash: &[u8]) -> Option<Transaction> {
    let mut vec_tx: Vec<Transaction> = Vec::new();

    for block in blocks {
        for tx in &block.txns {
            if tx.hash.into_inner() == hash {
                let last_tx = vec_tx.last();
                return last_tx.cloned();
            }
            vec_tx.push(tx.clone());
        }
    }

    None
}
