use std::str::FromStr;

use bitcoin_hashes::sha256::Hash as Sha256;
use bitcoin_hashes::{hash160, ripemd160, sha256, sha256d, Hash};

use secp256k1::{Secp256k1, SecretKey};

use crate::components::transaction::TransactionInput;
use crate::components::utxo_struct::Utxo;

pub fn u8_to_hex_string(slice: &[u8]) -> String {
    let hex_digits: Vec<String> = slice
        .iter()
        .rev()
        .map(|byte| format!("{:02X}", byte))
        .collect();

    hex_digits.join("")
}

pub fn serialize_var_int(value: u64) -> Vec<u8> {
    if value < 0xfd {
        vec![value as u8]
    } else if value <= 0xffff {
        let mut bytes = vec![0xfd];
        bytes.extend_from_slice(&(value as u16).to_le_bytes());
        return bytes;
    } else if value <= 0xffffffff {
        let mut bytes = vec![0xfe];
        bytes.extend_from_slice(&(value as u32).to_le_bytes());
        return bytes;
    } else {
        let mut bytes = vec![0xff];
        bytes.extend_from_slice(&value.to_le_bytes());
        return bytes;
    }
}

/// Auxiliar Function that reads bytes
pub fn read_var_int(data: &[u8]) -> Result<(u64, usize), &'static str> {
    let prefix_byte: u8 = data.first().cloned().ok_or("Insufficient data")?;
    match prefix_byte {
        0xfd => {
            let bytes = data.get(1..3).ok_or("Insufficient data")?;
            let value = u16::from_le_bytes([bytes[0], bytes[1]]) as u64;
            Ok((value, 3))
        }
        0xfe => {
            let bytes = data.get(1..5).ok_or("Insufficient data")?;
            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64;
            Ok((value, 5))
        }
        0xff => {
            let bytes = data.get(1..9).ok_or("Insufficient data")?;
            let value = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            Ok((value, 9))
        }
        _ => Ok((prefix_byte as u64, 1)),
    }
}

pub fn get_flag_value(data: &[u8]) -> bool {
    data[0] == 0x00 && data[1] == 0x01
}

pub fn find_spent_utxo(input: &TransactionInput, wallet_utxos: &Vec<Utxo>) -> Option<Utxo> {
    let txid_hash = u8_to_hex_string(&input.previous_output[0..32]);
    let _index = u32::from_be_bytes([
        input.previous_output[32],
        input.previous_output[33],
        input.previous_output[34],
        input.previous_output[35],
    ]);
    for utxo in wallet_utxos {
        if utxo.txid == txid_hash {
            return Some(utxo.clone());
        }
    }

    None
}

#[allow(dead_code)]
pub fn test_pubkey_to_address(address: &str) -> bool {
    let secp = Secp256k1::new();

    let private_key =
        SecretKey::from_str("FD792FBF1DCE3EFA84AC09CDAA564C19A7E749ABA797AABA3FC5787F370A4888")
            .unwrap();
    let public_key: [u8; 33] = private_key.public_key(&secp).serialize();
    let h160: [u8; 20] = hash160::Hash::hash(&public_key).into_inner();
    let version_prefix = [0x6f];
    let vec_to_hash: &Vec<u8> = &[&version_prefix[..], &h160[..]].concat();
    let hash1 = sha256::Hash::hash(vec_to_hash);
    let hash2 = sha256::Hash::hash(&hash1[..]);

    let checksum = &hash2[..4];
    let input = [&version_prefix[..], &h160[..], checksum].concat();
    let res = bs58::encode(input).into_string();
    res == address
}

pub fn hex_string_to_reversed_bytes_block_hash(
    hex: &str,
) -> Result<Vec<u8>, std::num::ParseIntError> {
    let mut bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()?;

    bytes.reverse();
    Ok(bytes)
}

pub fn reverse_hash(input: &[u8]) -> Vec<u8> {
    input.iter().rev().cloned().collect()
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02X}", byte)).collect()
}

pub fn address_from_script(script: &[u8]) -> Option<String> {
    match script.len() {
        20 => {
            // P2PKH
            // Address: version + hash160(public key) + checksum
            let mut address = vec![0x6f];
            address.extend_from_slice(script);
            let checksum = sha256d::Hash::hash(&address).into_inner()[..4].to_vec();
            address.extend_from_slice(&checksum);
            Some(bs58::encode(address).into_string())
        }
        22 if script.starts_with(&[0x00, 0x14]) => {
            // P2WPKH
            // Address: version + hash160(public key) + checksum
            None
        }
        23 if script.starts_with(&[0xa9, 0x14]) && script.ends_with(&[0x87]) => {
            // P2SH
            // Address: version + hash160(script) + checksum
            let mut address = vec![0xc4];
            address.extend_from_slice(&script[2..22]);
            let checksum = sha256d::Hash::hash(&address).into_inner()[..4].to_vec();
            address.extend_from_slice(&checksum);
            Some(bs58::encode(address).into_string())
        }
        25 if script.starts_with(&[0x76, 0xa9, 0x14]) && script.ends_with(&[0x88, 0xac]) => {
            // P2PKH
            // Address: version + hash160(public key) + checksum
            let mut address = vec![0x6f]; // version 0x6f for testnet
            address.extend_from_slice(&script[3..23]);

            // Calculate the double SHA256 hash of the result
            let checksum = sha256::Hash::hash(&sha256::Hash::hash(&address));
            // Take the first four bytes of step 2 as a checksum
            let checksum = &checksum[..4];
            // Append the checksum from step 3 to the result from step 1
            address.extend_from_slice(checksum);
            // Convert the final result to Base58
            Some(bs58::encode(address).into_string())
        }
        _ => None,
    }
}

pub fn get_value_after_keyword(input: &str, keyword: &str) -> Option<String> {
    if input.trim().starts_with(keyword) {
        return Some(input.trim().trim_start_matches(keyword).trim().to_string());
    }
    None
}

pub fn parse_inputs(inputs: Vec<String>) -> Vec<(Vec<u8>, Vec<u8>, u32)> {
    inputs
        .into_iter()
        .map(|input| {
            let split_input: Vec<&str> = input.split(',').collect();

            let previous_output_end = split_input
                .iter()
                .position(|&s| s.contains("script"))
                .unwrap();
            let script_end = split_input
                .iter()
                .position(|&s| s.contains("sequence"))
                .unwrap();

            let previous_output: Vec<u8> = split_input[0..previous_output_end]
                .iter()
                .map(|s| s.replace("previous_output:", "").trim().parse().unwrap())
                .collect();

            let script: Vec<u8> = split_input[previous_output_end..script_end]
                .iter()
                .filter(|s| **s != "[]" && !s.replace("script:", "").trim().is_empty())
                .map(|s| s.replace("script:", "").trim().parse().unwrap_or(0)) // default to 0 if parsing fails
                .collect();

            let sequence: u32 = split_input[script_end]
                .replace("sequence:", "")
                .trim()
                .parse()
                .unwrap();

            (previous_output, script, sequence)
        })
        .collect()
}

pub fn parse_output_values(outputs: Vec<String>) -> Vec<(u64, Vec<u8>)> {
    let mut result = Vec::new();

    for output in outputs {
        let split_output: Vec<&str> = output.split(", script_pubkey:").collect();

        let value: u64 = split_output[0]
            .trim_start_matches("value: ")
            .parse()
            .unwrap();

        let script_pubkey: Vec<u8> = split_output[1]
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();

        result.push((value, script_pubkey));
    }

    result
}

pub fn string_to_reversed_bytes(string: String) -> Vec<u8> {
    let bytes: Vec<u8> = string
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let byte_string = unsafe { std::str::from_utf8_unchecked(chunk) };
            u8::from_str_radix(byte_string, 16).unwrap()
        })
        .collect();

    bytes.into_iter().rev().collect()
}

pub fn split_ignore_brackets(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut inside_brackets = false;

    for c in s.chars() {
        match c {
            '[' => {
                inside_brackets = true;
                current.push(c);
            }
            ']' => {
                inside_brackets = false;
                current.push(c);
            }
            ',' => {
                if inside_brackets {
                    current.push(c);
                } else {
                    result.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

pub fn str_to_bytes(s: &str) -> Vec<u8> {
    s.trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|byte_str| byte_str.trim().parse().unwrap())
        .collect()
}
pub fn split_ignore_curly_brackets(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_item = String::new();
    let mut curly_brackets_count = 0;
    let mut square_brackets_count = 0;

    // Skip the first '{' character
    let s = s.strip_prefix('{').unwrap_or(s);

    for c in s.chars() {
        match c {
            '{' => curly_brackets_count += 1,
            '}' => curly_brackets_count -= 1,
            '[' => square_brackets_count += 1,
            ']' => square_brackets_count -= 1,
            ',' => {
                if curly_brackets_count == 0 && square_brackets_count == 0 {
                    result.push(current_item.clone());
                    current_item.clear();
                } else {
                    current_item.push(c);
                }
            }
            _ => current_item.push(c),
        }
    }
    if !current_item.is_empty() {
        result.push(current_item);
    }

    result
}

pub fn split_transaction_outputs(s: &str) -> Vec<String> {
    let split_string: Vec<&str> = s.split("TransactionOutput").collect();
    let mut result: Vec<String> = Vec::new();

    for item in &split_string[1..] {
        let trimmed_item = item.trim();
        if !trimmed_item.is_empty() {
            result.push(trimmed_item.to_string());
        }
    }

    result
}

pub fn split_transaction_inputs(s: &str) -> Vec<String> {
    let split_string: Vec<&str> = s.split("TransactionInput").collect();
    let mut result: Vec<String> = Vec::new();

    for item in &split_string[1..] {
        let trimmed_item = item.trim();
        if !trimmed_item.is_empty() {
            result.push(trimmed_item.to_string());
        }
    }

    result
}

#[allow(dead_code)]
pub fn process_outputs(s: &str) -> Vec<u32> {
    let value_str = s
        .split("value: ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap();
    let value = value_str.trim().parse().unwrap();

    let pubkey_str = s.split("script_pubkey: ").nth(1).unwrap().trim();
    let pubkey_vec: Vec<u32> = pubkey_str
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    // Construct the result vector
    let mut result = vec![value];
    result.extend(pubkey_vec);
    result
}

#[allow(dead_code)]
pub fn pubkey_to_address(pubkey: &[u8]) -> String {
    let sha = Sha256::hash(pubkey);
    let ripe: ripemd160::Hash = ripemd160::Hash::hash(&sha[..]);

    // Convertir el hash en una dirección de Bitcoin
    let mut address_payload = vec![0];
    address_payload.extend_from_slice(&ripe[..]);

    // Aplicar Base58Check
    let check_sum = &Sha256::hash(&Sha256::hash(&address_payload)[..])[0..4];
    address_payload.extend_from_slice(check_sum);
    let address = bs58::encode(address_payload).into_string();

    address
}

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in 0..hex.len() / 2 {
        let res = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => println!("Problem with hex: {}", e),
        }
    }

    bytes
}

pub fn hex_to_bytes_rev(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in 0..hex.len() / 2 {
        let res = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => println!("Problem with hex: {}", e),
        }
    }
    bytes.reverse();

    bytes
}

// script P2PKH
pub fn address_to_script_pubkey(address: &str) -> Vec<u8> {
    let mut decoded = bs58::decode(address).into_vec().unwrap();
    let hash_and_checksum = decoded.split_off(1);
    let hash160 = &hash_and_checksum[0..20]; // take hash160 part, ignore checksum
    let mut script = vec![];
    script.push(0x76); // OP_DUP
    script.push(0xa9); // OP_HASH160
    script.push(0x14); // OP_Data20
    script.extend(hash160); // add hash160
    script.push(0x88); // OP_EQUALVERIFY
    script.push(0xac); // OP_CHECKSIG
    script
}
