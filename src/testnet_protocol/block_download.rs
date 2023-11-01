use std::{
    io::{Error, ErrorKind, Read},
    net::{SocketAddr, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    components::{block::Block, block_header::BlockHeader, transaction::Transaction},
    connection::connection_protocol::set_tcp_stream_vec,
    interface::interfaz_grafica::{ChannelData, DownloadData},
    logger::{
        log_printer::{log_block, log_block_header},
        logger_impl::Logger,
    },
    testnet_protocol::{
        header_download::header_download,
        messages::{
            message_builders::build_get_data_message,
            message_parsers::{has_consecutive_zeros, parse_block, parse_transactions},
            message_senders::write_and_read_get_data_message,
        },
    },
};

pub fn initial_block_download(
    address: Vec<String>,
    node_sender: Arc<Mutex<gtk::glib::Sender<ChannelData>>>,
) -> Result<(), Error> {
    let tcp_stream_vec: Vec<TcpStream> = set_tcp_stream_vec(address.to_vec()).unwrap();

    let lista_seed_hashes: Vec<String> = vec![
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943".to_string(),
        "0000000000003739446e6bf168c07f87ee93a26ade070e8dfdae51ab20f33fa9".to_string(),
    ];

    let vec_tcp_localhost: Vec<TcpStream> = vec![tcp_stream_vec[0].try_clone().unwrap()];

    let node_sender_copy = Arc::clone(&node_sender);

    let lista_headers: Vec<BlockHeader> =
        header_download(&vec_tcp_localhost, lista_seed_hashes, node_sender_copy).unwrap();

    let logger_header = Logger::new("./logs", "headers_client")
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get logger"))
        .unwrap();

    println!("Cantidad de headers ---> {}", lista_headers.len());
    println!(
        "headers ---> {:?}",
        lista_headers[lista_headers.len() - 1].prev_block_hash
    );

    for header in lista_headers.clone() {
        log_block_header(Some(&logger_header), &header)?;
    }

    thread::sleep(Duration::from_secs(15));

    let mut headers_blocks: Vec<BlockHeader> = Vec::new();

    for header in lista_headers {
        if header.timestamp >= 1682380800 && header.timestamp <= 1688342400 {
            headers_blocks.push(header);
        }
    }

    println!(
        "Longitud de lista de headers que se van a pedir los blocks --->{}",
        headers_blocks.len()
    );

    let logger_block = Logger::new("./logs", "blocks_client")
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get logger"))?;

    let vec_tcp_copy: Vec<TcpStream> = vec![
        tcp_stream_vec[0].try_clone().unwrap(),
        tcp_stream_vec[1].try_clone().unwrap(),
        tcp_stream_vec[2].try_clone().unwrap(),
        tcp_stream_vec[3].try_clone().unwrap(),
    ];

    let lista_blocks: Vec<Block> =
        block_download(vec_tcp_copy, headers_blocks, node_sender).unwrap();

    println!(
        "Longitud blocks descargados del servidor ---> {}",
        lista_blocks.len()
    );

    for block in lista_blocks {
        log_block(Some(&logger_block), &block)?;
    }

    thread::sleep(Duration::from_secs(15));

    Ok(())
}

pub fn block_download(
    sockets: Vec<TcpStream>,
    lista_headers: Vec<BlockHeader>,
    node_sender: Arc<Mutex<gtk::glib::Sender<ChannelData>>>,
) -> Result<Vec<Block>, Error> {
    let len = lista_headers.len();
    let half_len = len / 2;
    let remainder_len = (len - half_len) / 3;

    let first_slice = lista_headers[0..half_len].to_vec();
    let second_slice = lista_headers[half_len..half_len + remainder_len].to_vec();
    let third_slice =
        lista_headers[half_len + remainder_len..half_len + 2 * remainder_len].to_vec();
    let fourth_slice = lista_headers[half_len + 2 * remainder_len..].to_vec();

    let divided_vecs = vec![first_slice, second_slice, third_slice, fourth_slice];

    let mut handles: Vec<JoinHandle<Vec<Block>>> = vec![];

    for i in 0..sockets.len() {
        let socket: TcpStream = sockets[i].try_clone().unwrap();

        let lista_headers_copia = divided_vecs[i].clone();

        let node_sender_copy = Arc::clone(&node_sender);

        let handle = thread::spawn(move || {
            let mut lista_blocks: Vec<Block> = vec![];

            for header in lista_headers_copia.clone() {
                let block_total = get_block_by_hash(&header.prev_block_hash, &socket);

                match block_total {
                    Ok(block) => {
                        lista_blocks.push(block.clone());
                    }
                    Err(_header) => {}
                }

                let node_sender_blocked = node_sender_copy.lock().unwrap();

                if socket.peer_addr().unwrap() == SocketAddr::from_str("127.0.0.1:18333").unwrap() {
                    let _ =
                        node_sender_blocked.send(ChannelData::DownloadDataBlocks(DownloadData {
                            total_data: lista_headers_copia.len() as f64,
                            received_data: lista_blocks.len() as f64,
                        }));
                }

                drop(node_sender_blocked);
            }
            lista_blocks
        });

        handles.push(handle);
    }

    let mut lista_blocks_total: Vec<Block> = Vec::new();

    let mut i = 0;
    let total_blocks_ignorados = 0;
    for handle in handles {
        match handle.join() {
            Ok(blocks) => {
                println!(
                    "Cantidad de bloques ignorados -> {}",
                    divided_vecs[i].len() - blocks.len()
                );
                i += 1;
                lista_blocks_total.extend(blocks);
            }
            Err(_) => println!("Hubo un error en descarga de blocks"),
        }
    }

    println!(
        "CANTIDAD TOTAL DE BLOCKS IGNORADOS ---> {}",
        total_blocks_ignorados
    );
    println!(
        "CANTIDAD DE BLOQUES A IMPRIMIR ---> {}",
        lista_blocks_total.len()
    );
    println!("CANTIDAD DE SUBLISTAS RESUELTAS ---> {}", i);

    Ok(lista_blocks_total)
}

pub fn get_block_by_hash(
    prev_block_hash: &[u8],
    mut socket: &TcpStream,
) -> Result<Block, BlockHeader> {
    let get_data_message = build_get_data_message(prev_block_hash).unwrap();

    let tuple_get_data_response = match write_and_read_get_data_message(&get_data_message, socket) {
        Ok(bytes_read_result) => bytes_read_result,
        Err(_header) => {
            return Err(BlockHeader::new(
                0,
                (0_u32).to_le_bytes().to_vec(),
                (0_u32).to_le_bytes().to_vec(),
                0,
                0,
                0,
            ));
        }
    };

    let payload = tuple_get_data_response.0;
    let long_buffer = tuple_get_data_response.1.len();

    println!(
        " ---------- LONGITUD DEL PAYLOAD EN EL HEDER DEL MESSAGE ---------- {}",
        payload
    );
    println!(
        " ---------- LONGITUD DEL BUFFER EN EL QUE ESTA EL PAYLOAD ---------- {}",
        long_buffer
    );

    println!(
        " ---------- SE RECIBIO EL HEADER DEL MENSAJE BLOCK, PASO A LEER EL PAYLOAD ---------- "
    );

    println!("Lectura post recibir HEADER de BLOCK message");

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

    let mut block_header = parse_block(response_buffer.clone());

    let mut response_buffer_aux = &response_buffer[..payload as usize];
    let mut tiene_ceros = has_consecutive_zeros(response_buffer_aux);

    let mut errores = 0;

    while !block_header.is_valid() || tiene_ceros {
        errores += 1;

        let get_data_message = build_get_data_message(prev_block_hash).unwrap();

        match write_and_read_get_data_message(&get_data_message, socket) {
            Ok(_) => {}
            Err(_header) => {
                return Err(BlockHeader::new(
                    0,
                    (0_u32).to_le_bytes().to_vec(),
                    (0_u32).to_le_bytes().to_vec(),
                    0,
                    0,
                    0,
                ));
            }
        }

        println!("parseando header mensaje block --->");

        println!("Lectura post recibir header de BLOCK message");

        response_buffer = vec![0; 500000];
        socket.read_exact(&mut response_buffer).unwrap();

        response_buffer_aux = &response_buffer[..payload as usize];
        tiene_ceros = has_consecutive_zeros(response_buffer_aux);

        block_header = parse_block(response_buffer.clone());

        if errores >= 6 {
            break;
        }
    }

    if errores >= 6 {
        return Err(block_header);
    }

    let transactions: Vec<Transaction> = parse_transactions(response_buffer).unwrap();

    let block = Block::new(block_header, transactions.len(), transactions);

    Ok(block)
}
