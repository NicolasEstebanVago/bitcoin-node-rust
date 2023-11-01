use std::{
    io::Error,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::ErrorKind::InvalidData;
use crate::{
    components::block::Block,
    helpers::auxiliar_functions::{read_var_int, u8_to_hex_string},
    logger::{log_printer::log_block, logger_impl::Logger},
    testnet_protocol::{
        block_download::get_block_by_hash,
        messages::{
            message_builders::build_tx_message, message_parsers::parse_message,
            message_senders::empty_socket,
        },
    },
};
pub fn broadcast_transaction(transaction_bytes: Vec<u8>, tcp_strema_vec: Vec<TcpStream>) {
    for mut socket in tcp_strema_vec {
        let tx_msg = build_tx_message(transaction_bytes.clone()).unwrap();
        let bytes_written_msg_tx = socket.write(&tx_msg).unwrap();
        println!(
            "TRANSACCION ENIVADA, LONGITUD DE LA ESCRITURA : {}",
            bytes_written_msg_tx
        );
    }
}

// Recibe un conjunto de structs tcpstreams, en los que ya se realizo el handshake
#[allow(dead_code)]
pub fn stablish_block_broadcasting(
    tcp_stream_vec: Vec<TcpStream>,
    sender: Arc<Mutex<Sender<Block>>>,
) {
    let mut handles: Vec<JoinHandle<()>> = vec![];

    for mut tcp_stream in tcp_stream_vec {
        let sender_hilo = sender.clone();

        let handle = thread::spawn(move || loop {
            println!("LISTENING FOR NEW INV MESSAGES ---> ");

            let mut response_buffer = [0; 100000];
            let read_size = tcp_stream.read(&mut response_buffer).unwrap();

            println!(
                " ----------- CANTIDAD DE BYTES LEIDOS ----------- {}",
                read_size
            );

            if read_size == 0 {
                break;
            }

            let (command, _payload) = parse_message(&response_buffer, None).unwrap();

            if command == *"inv" {
                let mut offset = 24;
                let (inventory_entries, bytes_ocupados) =
                    read_var_int(&response_buffer[offset..]).unwrap();

                offset += bytes_ocupados;

                for _ in 0..inventory_entries {
                    let hash_type = u32::from_le_bytes(
                        response_buffer[offset..offset + 4]
                            .try_into()
                            .map_err(|e| Error::new(InvalidData, e))
                            .unwrap(),
                    );

                    offset += 4;

                    println!("HASH TYPE DEL INV ---> {}", hash_type);

                    let inv_hash = &response_buffer[offset..offset + 32];
                    println!("    HASH DEL INV ---> : {:?}", u8_to_hex_string(inv_hash));

                    let logger_block = Logger::new("./logs", "blocks")
                        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get logger"))
                        .unwrap();

                    if hash_type == 2 {
                        let block: Block = get_block_by_hash(inv_hash, &tcp_stream).unwrap();
                        let _ = log_block(Some(&logger_block), &block);
                        let locked_sender = sender_hilo.lock().unwrap();
                        let _ = locked_sender.send(block);
                        drop(locked_sender);
                    }

                    offset += 32;
                }

                empty_socket(&mut tcp_stream);
            } else {
                empty_socket(&mut tcp_stream);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
