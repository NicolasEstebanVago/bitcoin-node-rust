use crate::components::block_header::BlockHeader;
use crate::helpers::auxiliar_functions::{
    bytes_to_hex, reverse_hash, string_to_reversed_bytes, u8_to_hex_string,
};
use crate::interface::interfaz_grafica::{ChannelData, DownloadData};
use crate::testnet_protocol::messages::message_builders::build_get_headers_message;
use crate::testnet_protocol::messages::message_parsers::parse_block_header;
use crate::testnet_protocol::messages::message_senders::write_and_read_get_headers_message;
use std::collections::HashSet;

use std::io::Error;
use std::net::TcpStream;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Creates the get headers response from the block with its parse function
pub fn header_download(
    sockets: &Vec<TcpStream>,
    lista_seed_hashes: Vec<String>,
    node_sender: Arc<Mutex<gtk::glib::Sender<ChannelData>>>,
) -> Result<Vec<BlockHeader>, Error> {
    // vector con los handlers de cada thread
    // JoinHandle<Vec<BlockHeader>>
    let mut handles: Vec<JoinHandle<Vec<BlockHeader>>> = vec![];

    for i in 0..sockets.len() {
        // se spawnea un hilo por cada socket que haya

        let hash_init: String = lista_seed_hashes[i].clone();
        let hash_stop: String = lista_seed_hashes[i + 1].clone();
        let mut socket: TcpStream = sockets[i].try_clone().unwrap();

        let node_sender_copy = Arc::clone(&node_sender);

        let handle = thread::spawn(move || {
            let mut prox_hash = string_to_reversed_bytes(hash_init);

            let mut set_hashes_descargados: HashSet<Vec<u8>> = HashSet::new();

            let mut firt_iterarion = true;
            let mut header_list: Vec<BlockHeader> = vec![];

            while !set_hashes_descargados.contains(&string_to_reversed_bytes(hash_stop.clone())) {
                let get_headers_msg = build_get_headers_message(prox_hash.clone()).unwrap();

                let response_buffer: Vec<u8> =
                    write_and_read_get_headers_message(&get_headers_msg, &mut socket).unwrap();

                let mut cursor = 3;

                for _i in 0..=1999 {
                    if response_buffer.is_empty() {
                        break;
                    }

                    let header: BlockHeader =
                        parse_block_header(&response_buffer[cursor..cursor + 81]).unwrap();

                    if header.is_valid() {
                        if !set_hashes_descargados.contains(&header.prev_block_hash) {
                            header_list.push(header.clone());
                        }
                    } else {
                        break;
                    }

                    if !firt_iterarion
                        && u8_to_hex_string(&header.prev_block_hash)
                            == *"000000000933EA01AD0EE984209779BAAEC3CED90FA3F408719526F8D77F4943"
                    {
                        break;
                    }

                    firt_iterarion = false;

                    if !set_hashes_descargados.contains(&header.prev_block_hash)
                        && header.is_valid()
                    {
                        set_hashes_descargados.insert(header.prev_block_hash.clone());
                    }

                    prox_hash = header.prev_block_hash.clone();

                    cursor += 81;
                }

                println!(
                    "CANTIDAD DE HEADERS DESCARGADOS ---> {}",
                    set_hashes_descargados.len()
                );

                let node_sender_blocked = node_sender_copy.lock().unwrap();

                let _ = node_sender_blocked.send(ChannelData::DownloadData(DownloadData {
                    total_data: 2400000.0,
                    received_data: set_hashes_descargados.len() as f64,
                }));

                drop(node_sender_blocked);

                println!(
                    "Ultimo hash ---> {}",
                    bytes_to_hex(&reverse_hash(&prox_hash))
                );
            }

            header_list
        });

        handles.push(handle);
    }

    let mut lista_headers_total: Vec<BlockHeader> = vec![];

    for handle in handles {
        match handle.join() {
            Ok(headers) => lista_headers_total.extend(headers),
            Err(_) => println!("Hubo un error en descarga de headers"),
        }
    }

    Ok(lista_headers_total)
}
