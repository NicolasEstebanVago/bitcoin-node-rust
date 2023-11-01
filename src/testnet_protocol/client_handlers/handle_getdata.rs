use std::{
    io::{Error, Write},
    net::TcpStream,
};
const BLOCK_MSG: &[u8; 12] = b"block\0\0\0\0\0\0\0";
const TX_MSG: &[u8; 12] = b"tx\0\0\0\0\0\0\0\0\0\0";
const NOTFOUND_MSG: &[u8; 12] = b"notfound\0\0\0\0";
const GET_DATA_MSG: &[u8; 12] = b"getdata\0\0\0\0\0";

use crate::{
    components::{block::Block, transaction::Transaction},
    helpers::{
        auxiliar_functions::{read_var_int, serialize_var_int},
        persistance::{get_blocks_from_memory, get_tx_from_memory},
    },
    testnet_protocol::messages::message_builders::build_header_message,
};

pub fn handle_getdata(buffer: &[u8], stream: &mut TcpStream, blocks: &Vec<Block>) {
    let mut offset = 24;
    let (inv_count, size) = read_var_int(&buffer[offset..]).unwrap();

    println!("INVCOUNT : {}", inv_count);
    offset += size;

    for _ in 0..inv_count {
        let inv_type = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
        offset += 4;
        println!("inv_type : {:?}", inv_type);

        let hash = &buffer[offset..offset + 32];
        offset += 32;
        println!("hash : {:?}", hash);

        match inv_type {
            2 => {
                let block = get_blocks_from_memory(blocks, hash);
                if let Some(block) = block {
                    let block_message = build_block_message(&block);
                    let _ = stream.write(&block_message);
                } else {
                    let not_found_message = build_not_found_message(inv_type, hash);
                    let _ = stream.write(&not_found_message);
                }
            }
            1 => {
                let tx: Option<Transaction> = get_tx_from_memory(blocks, hash);

                if let Some(tx) = tx {
                    println!(" TX {:?} ", tx.hash);
                    let tx_message = build_tx_message(&tx);
                    let _ = stream.write(&tx_message);
                } else {
                    let not_found_message = build_not_found_message(inv_type, hash);
                    let _ = stream.write(&not_found_message);
                }
            }
            _ => {
                let not_found_message = build_not_found_message(inv_type, hash);
                let _ = stream.write(&not_found_message);
            }
        }
    }
}

fn build_block_message(block: &Block) -> Vec<u8> {
    let mut payload: Vec<u8> = Vec::new();

    payload.extend_from_slice(&block.header.version.to_le_bytes());
    payload.extend_from_slice(&block.header.prev_block_hash);
    payload.extend_from_slice(&block.header.merkle_root);
    payload.extend_from_slice(&block.header.timestamp.to_le_bytes());
    payload.extend_from_slice(&block.header.bits.to_le_bytes());
    payload.extend_from_slice(&block.header.nonce.to_le_bytes());

    let txn_count = block.txn_count as u64;
    payload.extend_from_slice(&serialize_var_int(txn_count));

    for tx in &block.txns {
        payload.extend_from_slice(&serialize_tx(tx));
    }

    let mut header = build_header_message(payload.clone(), BLOCK_MSG);

    header.extend_from_slice(&payload);
    header
}

fn serialize_tx(tx: &Transaction) -> Vec<u8> {
    let mut payload = Vec::new();

    payload.extend_from_slice(&tx.version.to_le_bytes());
    payload.extend_from_slice(&serialize_var_int(tx.tx_in_count as u64));

    for input in &tx.inputs {
        payload.extend_from_slice(&input.previous_output);
        payload.extend_from_slice(&serialize_var_int(input.script.len() as u64));
        payload.extend_from_slice(&input.script);
        payload.extend_from_slice(&input.sequence.to_le_bytes());
    }

    payload.extend_from_slice(&serialize_var_int(tx.tx_out_count as u64));

    for output in &tx.outputs {
        payload.extend_from_slice(&output.value.to_le_bytes());
        payload.extend_from_slice(&serialize_var_int(output.script_pubkey.len() as u64));
        payload.extend_from_slice(&output.script_pubkey);
    }

    payload.extend_from_slice(&tx.lock_time.to_le_bytes());

    payload
}

fn build_tx_message(tx: &Transaction) -> Vec<u8> {
    let payload = serialize_tx(tx);
    let mut header = build_header_message(payload.clone(), TX_MSG);

    header.extend_from_slice(&payload);
    header
}

fn build_not_found_message(inv_type: u32, hash: &[u8]) -> Vec<u8> {
    let mut payload: Vec<u8> = Vec::new();

    let num_inv_vect: u8 = 1;
    payload.extend_from_slice(&num_inv_vect.to_le_bytes());

    payload.extend_from_slice(&inv_type.to_le_bytes());
    payload.extend_from_slice(hash);

    let mut header = build_header_message(payload.clone(), NOTFOUND_MSG);

    header.extend_from_slice(&payload);

    header
}

pub fn build_get_data_message_tx(prev_block_hash: &[u8]) -> Result<Vec<u8>, Error> {
    println!("Inside build_get_data_message");

    let mut payload: Vec<u8> = Vec::new();

    // Number of inventory vectors, 1 byte (en este caso pedimos 1 solo bloque)
    let num_inv_vect: u8 = 1;
    payload.extend_from_slice(&num_inv_vect.to_le_bytes());

    let object_type: u32 = 1;
    // Inventory vector, 36 bytes each (4 for type, 32 for hash)
    payload.extend_from_slice(&object_type.to_le_bytes()); // investigar

    // let block_hash: Vec<u8> = prev_block_hash.to_owned(); // sin revertirlos

    let block_hash: Vec<u8> = prev_block_hash.to_owned();
    payload.extend_from_slice(&block_hash);
    // Construct the header
    let mut header = build_header_message(payload.clone(), GET_DATA_MSG);

    // Combine header and payload
    header.extend_from_slice(&payload);

    Ok(header)
}
