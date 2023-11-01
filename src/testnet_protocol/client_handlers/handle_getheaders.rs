use std::{io::Write, net::TcpStream};

use crate::{
    components::block_header::BlockHeader,
    helpers::{auxiliar_functions::read_var_int, persistance::get_headers_from_memory},
    testnet_protocol::messages::message_builders::build_header_message,
};

pub fn handle_getheaders(buffer: &[u8], stream: &mut TcpStream, headers: &Vec<BlockHeader>) {
    let mut offset = 28;
    let (_hash_count, size) = read_var_int(&buffer[offset..]).unwrap();
    offset += size;
    let hash = &buffer[offset..offset + 32];
    offset += 32;

    let hash_stop = &buffer[offset..offset + 32];

    println!(" HASH START {:?}", hash);

    let headers = get_headers_from_memory(headers, hash, hash_stop);
    println!(" lenght headers recieved {}", headers.len());

    let headers_message = build_headers_message(headers);
    let bytes_written = stream.write(&headers_message);

    println!(" bytes written {:?}", bytes_written.unwrap());
}

pub fn build_headers_message(headers: Vec<BlockHeader>) -> Vec<u8> {
    let mut payload: Vec<u8> = Vec::new();
    let _header_count = headers.len() as u64;
    payload.push(0x01);
    payload.push(0x02);
    payload.push(0x03);

    for header in headers {
        let serialized_header = serialize_block_header(&header);
        payload.extend_from_slice(&serialized_header);
        payload.push(0x00);
    }

    let mut header = build_header_message(payload.clone(), b"headers\0\0\0\0\0");

    header.extend_from_slice(&payload);
    header
}

fn serialize_block_header(header: &BlockHeader) -> Vec<u8> {
    let mut serialized = Vec::new();
    serialized.extend_from_slice(&header.version.to_le_bytes());
    serialized.extend_from_slice(&header.prev_block_hash);
    serialized.extend_from_slice(&header.merkle_root);
    serialized.extend_from_slice(&header.timestamp.to_le_bytes());
    serialized.extend_from_slice(&header.bits.to_le_bytes());
    serialized.extend_from_slice(&header.nonce.to_le_bytes());
    serialized
}
