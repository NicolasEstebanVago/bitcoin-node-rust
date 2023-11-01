use crate::{
    components::{
        block_header::BlockHeader,
        transaction::{Transaction, TransactionInput, TransactionOutput},
    },
    helpers::auxiliar_functions::{get_flag_value, read_var_int, u8_to_hex_string},
    logger::logger_impl::Logger,
};
use bitcoin_hashes::{sha256d, Hash};
use std::io::{Error, ErrorKind};

use crate::ErrorKind::InvalidData;

fn sha256d(buffer: &[u8]) -> sha256d::Hash {
    sha256d::Hash::hash(buffer)
}

pub fn has_consecutive_zeros(arr: &[u8]) -> bool {
    let mut count_zeros = 0;

    for &num in arr {
        if num == 0 {
            count_zeros += 1;
        } else {
            count_zeros = 0;
        }

        if count_zeros == 50 {
            return true;
        }
    }

    false
}

pub fn parse_tx_out(
    tx_out_count: Result<(u64, usize), &str>,
    response_buffer: Vec<u8>,
    mut offset: usize,
) -> Result<(Vec<TransactionOutput>, usize), Error> {
    let mut tx_out_list = Vec::new();
    let tx_out_count_value = tx_out_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .0;

    for _ in 0..tx_out_count_value {
        let value_bytes = &response_buffer[offset..offset + 8];
        let value_bytes_array: [u8; 8] = value_bytes
            .try_into()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        let value = u64::from_le_bytes(value_bytes_array);

        offset += 8;

        let pk_script_length = read_var_int(&response_buffer[offset..]);

        offset += pk_script_length
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .1;

        let pk_script = &response_buffer[offset
            ..offset
                + (pk_script_length
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
                    .0 as usize)];

        offset += pk_script_length
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .0 as usize;

        let tx_out = TransactionOutput {
            value,
            script_pubkey: pk_script.to_vec(),
        };

        tx_out_list.push(tx_out);
    }
    Ok((tx_out_list, offset))
}

pub fn parse_tx_in(
    tx_in_count: Result<(u64, usize), &str>,
    response_buffer: Vec<u8>,
    mut offset: usize,
) -> Result<(Vec<TransactionInput>, usize), Error> {
    let mut tx_in_list = Vec::new();
    let tx_in_count_value = tx_in_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .0;
    for _ in 0..tx_in_count_value {
        let mut previous_output: [u8; 36] = [0; 36];

        let prev_block_hash = &response_buffer[offset..offset + 32];

        previous_output[..32].copy_from_slice(&response_buffer[offset..offset + 32]);
        offset += 32;

        let bytes_index = &response_buffer[offset..offset + 4];
        let bytes_array_index: [u8; 4] = bytes_index
            .try_into()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let index = u32::from_le_bytes(bytes_array_index);

        previous_output[32..].copy_from_slice(&response_buffer[offset..offset + 4]);
        offset += 4;

        if index == 4294967295 && prev_block_hash.iter().all(|&x| x == 0) {
            println!("    THIS is a coinbase transaction!\n");
        }

        let script_length = read_var_int(&response_buffer[offset..]);

        offset += script_length
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .1;

        let sig_script = &response_buffer[offset
            ..offset
                + (script_length
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
                    .0 as usize)];

        offset += script_length
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .0 as usize;

        let bytes_sequence = &response_buffer[offset..offset + 4];
        let bytes_array_sequence: [u8; 4] = bytes_sequence
            .try_into()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let sequence = u32::from_le_bytes(bytes_array_sequence);

        offset += 4;

        let tx_in = TransactionInput {
            previous_output,
            script: sig_script.to_vec(),
            sequence,
        };

        tx_in_list.push(tx_in);
    }
    Ok((tx_in_list, offset))
}

pub fn get_witness(response_buffer: Vec<u8>, mut offset: usize) -> Result<usize, Error> {
    let witness_count = read_var_int(&response_buffer[offset..]);

    offset += witness_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .1;

    for _ in 0..witness_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .0
    {
        let witness_length = read_var_int(&response_buffer[offset..]);

        offset += witness_length
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .1;

        offset += witness_length
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .0 as usize;
    }

    Ok(offset)
}

pub fn parse_block(response_buffer: Vec<u8>) -> BlockHeader {
    println!("Parseando el PAYLOAD DE BLOCK ---> ");
    let version = u32::from_le_bytes([
        response_buffer[0],
        response_buffer[1],
        response_buffer[2],
        response_buffer[3],
    ]);
    println!("    BLOCK Version --->  {}", version);

    let prev_block_hash = &response_buffer[4..36];
    let merkle_root = &response_buffer[36..68];
    println!(
        "    BLOCK Previous block hash: {:?}",
        u8_to_hex_string(prev_block_hash)
    );
    println!("    BLOCK Merkle root: {:?}", u8_to_hex_string(merkle_root));

    let timestamp = u32::from_le_bytes([
        response_buffer[68],
        response_buffer[69],
        response_buffer[70],
        response_buffer[71],
    ]);
    println!("    BLOCK Timestamp --->  {}", timestamp);

    let bits = u32::from_le_bytes([
        response_buffer[72],
        response_buffer[73],
        response_buffer[74],
        response_buffer[75],
    ]);
    println!("    BLOCK Bits --->  {}", bits);

    let nonce = u32::from_le_bytes([
        response_buffer[76],
        response_buffer[77],
        response_buffer[78],
        response_buffer[79],
    ]);
    println!("    BLOCK nonce --->  {}", nonce);

    BlockHeader::new(
        version,
        prev_block_hash.to_vec(),
        merkle_root.to_vec(),
        timestamp,
        bits,
        nonce,
    )
}

pub fn parse_transaction(
    response_buffer: Vec<u8>,
    offset: &mut usize,
) -> Result<Transaction, Error> {
    let bytes = &response_buffer[*offset..*offset + 4];
    let bytes_array: [u8; 4] = bytes
        .try_into()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    let transaction_version = u32::from_le_bytes(bytes_array);

    let initial_offset = *offset;

    *offset += 4;

    let transaction_flag = get_flag_value(&response_buffer[*offset..]);

    if transaction_flag {
        println!("    Tiene witness, +2 en offset");
        *offset += 2;
    }

    let tx_in_count = read_var_int(&response_buffer[*offset..]);

    *offset += tx_in_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .1;

    let (tx_in_list, _offset) = parse_tx_in(tx_in_count, response_buffer.clone(), *offset)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    *offset = _offset;

    let tnx_out_count = read_var_int(&response_buffer[*offset..]);

    *offset += tnx_out_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .1;
    let (tx_out_list, _offset) = parse_tx_out(tnx_out_count, response_buffer.clone(), *offset)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    *offset = _offset;

    if transaction_flag {
        let _ = get_witness(response_buffer.clone(), *offset);
    }

    let bytes_locktime = &response_buffer[*offset..*offset + 4];
    let bytes_array_locktime: [u8; 4] = bytes_locktime
        .try_into()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    let lock_time = u32::from_le_bytes(bytes_array_locktime);

    *offset += 4;

    let raw_hash = sha256d(&response_buffer[initial_offset..*offset]);
    let mut id = raw_hash.to_vec();
    id.reverse(); // reverse the bytes to get the transaction id

    println!("    TRANSACTION HASH --->  {} \n", raw_hash);
    let transaction = Transaction {
        hash: raw_hash,
        version: transaction_version,
        tx_in_count: tx_in_count
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .0 as u32,
        inputs: tx_in_list,
        tx_out_count: tnx_out_count
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .0 as u32,
        outputs: tx_out_list,
        lock_time,
        txid: id,
    };

    Ok(transaction)
}

pub fn parse_transactions(response_buffer: Vec<u8>) -> Result<Vec<Transaction>, Error> {
    let tnx_count: Result<(u64, usize), &str> = read_var_int(&response_buffer[80..]);
    println!(
        "    CANTIDAD DE TRANSACCIONES RECIBIDAS {} ",
        tnx_count
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .0
    );

    let mut transactions = Vec::<Transaction>::new();

    let mut offset = 80
        + tnx_count
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .1;

    for _ in 0..tnx_count
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .0
    {
        let transaction = parse_transaction(response_buffer.clone(), &mut offset)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        transactions.push(transaction);
    }

    Ok(transactions)
}

/// Parses the header version message
pub fn parse_message(buffer: &[u8], logger: Option<&Logger>) -> Result<(String, u32), Error> {
    if let Some(logger) = logger {
        logger.log("")?;
    }

    let magic = u32::from_le_bytes(
        buffer[..4]
            .try_into()
            .map_err(|e| Error::new(InvalidData, e))?,
    );
    let command = String::from_utf8_lossy(&buffer[4..16]);
    let length = u32::from_le_bytes(
        buffer[16..20]
            .try_into()
            .map_err(|e| Error::new(InvalidData, e))?,
    );
    let checksum = &buffer[20..24];

    println!("Message:");
    println!("    Magic: {}", magic);
    println!("    COMMAND: {}", command.trim_end_matches('\x00'));
    println!("    PAYLOAD LENGHT: {}", length);
    println!("    Checksum: {:?}", checksum);

    Ok((command.trim_end_matches('\x00').to_string(), length))
}

/// Parses the block header so the data is seen properly
pub fn parse_block_header(block_header: &[u8]) -> Result<BlockHeader, Error> {
    let version = u32::from_le_bytes(
        block_header[0..4]
            .try_into()
            .map_err(|e| Error::new(InvalidData, e))?,
    );

    let prev_block_hash = &block_header[4..36];
    let merkle_root = &block_header[36..68];
    let timestamp = u32::from_le_bytes(
        block_header[68..72]
            .try_into()
            .map_err(|e| Error::new(InvalidData, e))?,
    );

    let bits = u32::from_le_bytes(
        block_header[72..76]
            .try_into()
            .map_err(|e| Error::new(InvalidData, e))?,
    );

    let nonce = u32::from_le_bytes(
        block_header[76..80]
            .try_into()
            .map_err(|e| Error::new(InvalidData, e))?,
    );

    let block_header = BlockHeader::new(
        version,
        prev_block_hash.to_vec(),
        merkle_root.to_vec(),
        timestamp,
        bits,
        nonce,
    );

    Ok(block_header)
}
