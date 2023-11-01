use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpStream},
    thread,
    time::Duration,
    vec,
};

use crate::{
    components::{block::Block, transaction::Transaction},
    helpers::auxiliar_functions::hex_to_bytes,
    testnet_protocol::messages::{
        message_builders::{build_get_data_message, build_get_headers_message},
        message_parsers::{parse_block, parse_block_header, parse_message, parse_transactions},
        message_senders::{write_and_read_get_data_message, write_and_read_get_headers_message},
    },
};

use super::handle_getdata::build_get_data_message_tx;

#[allow(dead_code)]
pub fn testing_getheaders(tcp_stream_vec: &mut Result<Vec<TcpStream>, Error>) {
    if let Ok(ref mut vec) = *tcp_stream_vec {
        let local_host_stream = vec.iter_mut().find(|stream| match stream.peer_addr() {
            Ok(addr) => addr == SocketAddr::from(([127, 0, 0, 1], 18333)),
            Err(_) => false,
        });

        let get_headers_msg = build_get_headers_message(hex_to_bytes(
            "000000000058b74204bb9d59128e7975b683ac73910660b6531e59523fb4a102",
        ))
        .unwrap();

        let mut _response_buffer: Vec<u8> = Vec::new();
        if let Some(stream) = local_host_stream {
            _response_buffer =
                write_and_read_get_headers_message(&get_headers_msg, stream).unwrap();
        }

        let mut vec = Vec::new();
        let mut offset = 0;
        while offset <= 400 {
            let header = parse_block_header(&_response_buffer[offset..offset + 80]).unwrap();
            vec.push(header);
            offset += 80;
        }
    }
}

#[allow(dead_code)]
pub fn testing_getdata_block(tcp_stream_vec: &mut Result<Vec<TcpStream>, Error>) {
    if let Ok(ref mut vec) = *tcp_stream_vec {
        let local_host_stream = vec.iter_mut().find(|stream| match stream.peer_addr() {
            Ok(addr) => addr == SocketAddr::from(([127, 0, 0, 1], 18333)),
            Err(_) => false,
        });

        let get_data_message = build_get_data_message(&hex_to_bytes(
            "000000000000000b111111ecf409006d6dc863371d9aed3159040419a9eb1c48",
        ))
        .unwrap();

        let mut _response_buffer: (u32, Vec<u8>) = (0, Vec::new());

        if let Some(stream) = local_host_stream {
            _response_buffer = write_and_read_get_data_message(&get_data_message, stream).unwrap();
            println!(
                "response buffer {:?}",
                parse_block_response(_response_buffer, stream)
            );
        }
    }
}

#[allow(dead_code)]
pub fn testing_getdata_tx(tcp_stream_vec: &mut Result<Vec<TcpStream>, Error>) {
    if let Ok(ref mut vec) = *tcp_stream_vec {
        let local_host_stream = vec.iter_mut().find(|stream| match stream.peer_addr() {
            Ok(addr) => addr == SocketAddr::from(([127, 0, 0, 1], 18333)),
            Err(_) => false,
        });

        let get_data_message = build_get_data_message_tx(&hex_to_bytes(
            "B2B6CEC9077100C6433F3C5F88157DD45A92BF5A9F8222403693466AF70A9AAE",
        ))
        .unwrap();

        let mut _response_buffer: (u32, Vec<u8>) = (0, Vec::new());

        if let Some(stream) = local_host_stream {
            _response_buffer =
                write_and_read_get_data_message_tx(&get_data_message, stream).unwrap();
            println!("SE RECIBIO UNA TX ");
        }
    }
}

pub fn write_and_read_get_data_message_tx(
    get_data_message: &[u8],
    mut socket: &TcpStream,
) -> Result<(u32, Vec<u8>), String> {
    let _bytes_written = socket.write(get_data_message).unwrap();

    let mut response_buffer = [0; 100000];
    let mut response_size = socket.read(&mut response_buffer).unwrap();

    println!(
        " ---------- Tamaño de lectura post enviar get data ---------- {} ",
        response_size
    );

    let (mut command, mut payload) = parse_message(&response_buffer, None).unwrap();

    let mut errores = 0;

    while command != *"tx" && command != *"notfound" {
        errores += 1;

        if errores == 15 {
            return Err("Error en mensaje getdata".to_string());
        }

        thread::sleep(Duration::from_millis(350));

        let mut _response_buffer_aux = [0; 5000];
        let mut _response_size_aux = socket.read(&mut _response_buffer_aux).unwrap();

        while _response_size_aux == 5000 {
            _response_buffer_aux = [0; 5000];
            _response_size_aux = socket.read(&mut _response_buffer_aux).unwrap();
        }

        socket.write_all(get_data_message).unwrap();

        response_buffer = [0; 100000];
        response_size = socket.read(&mut response_buffer).unwrap();

        println!(
            " ---------- Tamaño de lectura post enviar get data ---------- {} ",
            response_size
        );
        (command, payload) = parse_message(&response_buffer, None).unwrap();
        if command == *"notfound" {
            return Ok((payload, response_buffer[..response_size].to_vec()));
        }
    }

    Ok((payload, response_buffer[..response_size].to_vec()))
}

pub fn parse_block_response(buffer: (u32, Vec<u8>), mut socket: &TcpStream) -> Block {
    let tuple_get_data_response: (u32, Vec<u8>) = buffer;
    let payload = tuple_get_data_response.0;
    let long_buffer = tuple_get_data_response.1.len();

    let mut response_buffer: Vec<u8> = Vec::new();

    if long_buffer > 24 {
        response_buffer.extend_from_slice(&tuple_get_data_response.1[24..]);
    }

    while response_buffer.len() < payload.try_into().unwrap() {
        //
        let mut response_buffer_aux = vec![0; 500000];
        let read_size_payload_block_message = socket.read(&mut response_buffer_aux).unwrap();
        response_buffer.extend_from_slice(&response_buffer_aux[..read_size_payload_block_message]);
    }

    let block_header = parse_block(response_buffer.clone());

    let transactions: Vec<Transaction> = parse_transactions(response_buffer).unwrap();

    Block::new(block_header, transactions.len(), transactions)
}
