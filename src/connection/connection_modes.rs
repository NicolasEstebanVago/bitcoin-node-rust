use std::{
    collections::HashMap,
    fs::File,
    io::{self, Error, Read},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use gio::glib;
use secp256k1::SecretKey;

use crate::{
    components::{
        block::Block,
        block_header::BlockHeader,
        user::{is_tx_valid_in_block, User},
        wallet::{update_wallet, Wallet},
    },
    connection::connection_protocol::handshake_server,
    helpers::persistance::{get_blocks_from_file, get_headers_from_file},
    interface::interfaz_grafica::{
        interfaz, BalanceData, ChannelData, DownloadData, TransactionData,
    },
    testnet_protocol::{
        block_download::initial_block_download,
        client_handlers::{handle_getdata::handle_getdata, handle_getheaders::handle_getheaders},
        messages::message_parsers::parse_message,
    },
};

use super::connection_protocol::fetch_nodes_config;

#[allow(dead_code)]
fn handle_user_interface(
    wallet: &mut Wallet,
    node_sender: Arc<Mutex<gtk::glib::Sender<ChannelData>>>,
) {
    // Se bloquea el node sender
    let node_sender_blocked = node_sender.lock().unwrap();

    let balance: ChannelData = ChannelData::Balance(BalanceData {
        available: wallet.calculate_balance().to_string(),
        pending: 0.to_string(),
        inmature: 0.to_string(),
        total: 0.to_string(),
    });

    node_sender_blocked
        .send(balance)
        .expect("error en send balance data");

    //let transactions: ChannelData = get_new_transactions();
    let transactions = wallet.get_transactions_history();
    let mut vec_transactions = Vec::new();

    for transaction in transactions {
        let transaction = TransactionData {
            status: "1".to_string(),
            date: "26/6/23".to_string(),
            tipe: "Mined".to_string(),
            label: wallet.address.to_string(),
            amount: transaction.get_amount(wallet).to_string(),
        };
        vec_transactions.push(transaction);
    }

    let transaction: ChannelData = ChannelData::Transactions(vec_transactions);
    node_sender_blocked
        .send(transaction)
        .expect("error en send refresh transactions");

    // Se desbloquea el node sender
    drop(node_sender_blocked);
}

#[allow(dead_code)]
fn spawn_block_reciv_thread(
    address: String,
    hash_table_addresses: Arc<Mutex<HashMap<String, Wallet>>>,
    receiver: Arc<Mutex<Receiver<Block>>>,
    node_sender: Arc<Mutex<gtk::glib::Sender<ChannelData>>>,
) -> std::thread::JoinHandle<()> {
    let copy_reciver = receiver; // copio y paso a un thread
    thread::spawn(move || {
        let reciver_blocks = copy_reciver.lock().unwrap();

        loop {
            let block_new = reciver_blocks.recv().unwrap(); // se queda trabado este hilo, esperando a recbir luz verde para actualizar la wallet con el nuevo block
                                                            // update_wallet(&mut wallet, block_new.clone());
            println!("Se recibio un nuevo bloque...");

            let mut hashtable_wallets_blocked = hash_table_addresses.lock().unwrap();

            let wallet = hashtable_wallets_blocked.get_mut(&address).unwrap();

            update_wallet(wallet, block_new);

            handle_user_interface(wallet, node_sender.clone()); // enviando a la interfaz los nuevos datos

            drop(hashtable_wallets_blocked);
        }
    })
}

fn handle_recived_data(
    data: ChannelData,
    hashtable_wallets: Arc<Mutex<HashMap<String, Wallet>>>,
    node_sender: Arc<Mutex<glib::Sender<ChannelData>>>,
) -> Result<(), io::Error> {
    match data {
        ChannelData::Account(account_info) => {
            println!("Estoy recibiendo los siguientes datos");
            println!("Nombre: {}", account_info.name);
            println!("Address: {}", account_info.address);
            println!("Private key: {}", account_info.private_key);

            let private_key = SecretKey::from_str(&account_info.private_key).unwrap();

            let mut user = User::new("Nico".to_owned());
            user.create_new_wallet(&private_key, &account_info.address);
            let _reader = io::BufReader::new(
                File::open("logs/blocks.txt").expect("Error to open logs/blocks"),
            );

            // let lista_blocks = get_blocks_from_memory(reader, );

            // for mut wallet in user.get_wallets() {
            //     // let validated_blocks = get_valid_blocks(lista_blocks.clone());
            //     for validated_block in &lista_blocks {
            //         update_wallet(&mut wallet, validated_block.clone());
            //     }
            // }

            let mut hashtable_wallets_blocked = hashtable_wallets.lock().unwrap();

            hashtable_wallets_blocked.insert(
                account_info.address.to_string(),
                user.get_wallets()[0].clone(),
            );

            let _wallet = hashtable_wallets_blocked
                .get_mut(&account_info.address.to_string())
                .unwrap();

            //handle_user_interface(wallet, node_sender.clone()); // enviando a la interfaz los nuevos datos

            drop(hashtable_wallets_blocked);

            // spawn_block_reciv_thread(
            //     account_info.address.to_string(),
            //     hashtable_wallets.clone(),
            //     receiver.clone(),
            //     node_sender.clone(),
            // );
        }
        ChannelData::Payment(sender_payment) => {
            println!("Estoy recibiendo los siguientes datos");
            println!("Address: {}", sender_payment.address);
            println!("Nombre: {}", sender_payment.amount);

            let mut hashtable_wallets_blocked = hashtable_wallets.lock().unwrap();

            hashtable_wallets_blocked
                .get_mut(&sender_payment.own_address)
                .unwrap()
                .create_transaction(&sender_payment.address, sender_payment.amount as u64);

            drop(hashtable_wallets_blocked);
        }
        ChannelData::EndInterface => {
            println!("Finaliza el hilo secundario");
            return Err(io::Error::new(io::ErrorKind::Other, "End of interface"));
        }
        ChannelData::ProofOfInclusion(data) => {
            println!(" NODO  {}, {}", data.block_hash, data.transaction_hash);

            let _result = is_tx_valid_in_block(data.transaction_hash, data.block_hash);

            // let node_sender_blocked = node_sender.lock().unwrap();

            // node_sender_blocked
            //     .send(ChannelData::ResponseProofOfInclusion(result))
            //     .expect("error en proof of inclusion");

            // drop(node_sender_blocked);
        }
        ChannelData::RequestDownload => {
            println!("Server recibe RequestDownload");

            let node_sender_blocked = node_sender.lock().unwrap();

            let total_download = 100.00;
            let mut data_sent = 0.00;

            while data_sent <= 100.00 {
                thread::sleep(std::time::Duration::new(2, 0));

                let blocks_to_send = 10.00;

                let download_data: ChannelData = ChannelData::DownloadData(DownloadData {
                    total_data: total_download,
                    received_data: blocks_to_send,
                });

                node_sender_blocked
                    .send(download_data)
                    .expect("Error en send DownloadData to interface");

                data_sent += blocks_to_send;
            }

            println!("END OF REQUEST DOWNLOAD----------------")
        }
        _ => (),
    };

    Ok(())
}

pub fn client_mode() {
    let (node_sender, interface_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (interface_sender, node_receiver): (Sender<ChannelData>, Receiver<ChannelData>) = channel();

    let node_sender = Arc::new(Mutex::new(node_sender));

    let node_sender_copy = Arc::clone(&node_sender);

    let _ibd_thread = thread::spawn(move || {
        println!("Comenzando descarga en hilo descarga...");
        let nodes = fetch_nodes_config().expect("Error in fetching nodes");

        initial_block_download(nodes[0..6].to_vec(), node_sender_copy).unwrap();
    });

    let node_thread = thread::spawn(move || {
        let (sender, receiver) = channel::<Block>(); // se puede no usar

        let _sender = Arc::new(Mutex::new(sender)); // se puede no usar
        let _receiver = Arc::new(Mutex::new(receiver));

        // let thread_broadcasting = thread::spawn(move || {
        //     stablish_block_broadcasting(tcp_stream_vec.unwrap(), sender); // en su propio hilo, para que se qeude trabado ese hilo
        // });

        let hashtable_wallets_: HashMap<String, Wallet> = HashMap::new();
        let hashtable_wallets: Arc<Mutex<HashMap<String, Wallet>>> =
            Arc::new(Mutex::new(hashtable_wallets_));

        loop {
            if let Ok(data) = node_receiver.try_recv() {
                println!("SE RECIBIO ALGO DE LA INTERFAZ");
                if handle_recived_data(data, hashtable_wallets.clone(), node_sender.clone())
                    .is_err()
                {
                    break;
                }
            };
        }
        // thread_broadcasting.join().unwrap();
    });

    gtk::init().expect("Error to init GTK");

    interfaz(interface_sender, interface_receiver);

    println!("Antes de ejecutar gtk::main");
    gtk::main();

    println!("Aplicación finalizada");
    node_thread.join().expect("Other thread panicked.");
}

pub fn server_mode() -> Result<(), Error> {
    println!("Cargando files en memoria, esperar a ser avisado para correr el Cliente...");
    println!("La carga de files en memoria puede tardar unos minutos...");

    let listener = TcpListener::bind("0.0.0.0:18333")?;
    let reader_headers = io::BufReader::new(File::open("logs/headers.txt").unwrap());
    let headers: Vec<BlockHeader> = get_headers_from_file(reader_headers);
    let headers = Arc::new(headers);

    let reader_blocks = io::BufReader::new(File::open("logs/blocks.txt").unwrap());
    let blocks = get_blocks_from_file(reader_blocks);
    let blocks: Arc<Vec<Block>> = Arc::new(blocks);

    println!(" SE DESCARGARON DEL FILE {} headers", headers.len());
    println!(" SE DESCARGARON DEL FILE {} bloques", blocks.len());

    println!("Ya se puede iniciar el Cliente...");

    println!("Servidor escuchando conexiones...");

    for stream in listener.incoming() {
        handle_response(stream, headers.clone(), blocks.clone());
    }

    Ok(())
}

fn handle_response(
    stream: Result<TcpStream, Error>,
    headers: Arc<Vec<BlockHeader>>,
    blocks: Arc<Vec<Block>>,
) {
    match stream {
        Ok(mut stream) => {
            println!(" recibo conexion");

            let headers = Arc::clone(&headers);
            let blocks = Arc::clone(&blocks);

            let handle = thread::spawn(move || loop {
                let mut response_buffer = [0; 100000];
                let read_size = stream.read(&mut response_buffer).unwrap();

                if read_size == 0 {
                    break;
                }

                let (command, _payload) = parse_message(&response_buffer, None).unwrap();
                handle_command(
                    command,
                    stream.try_clone().unwrap(),
                    headers.clone(),
                    blocks.clone(),
                    &response_buffer,
                );
            });
            handle.join().unwrap();
        }
        Err(_) => println!(" error"),
    }
}

fn handle_command(
    command: String,
    mut stream: TcpStream,
    headers: Arc<Vec<BlockHeader>>,
    blocks: Arc<Vec<Block>>,
    buffer: &[u8],
) {
    match command.as_str() {
        "version" => {
            println!(" SE RECIBE VERSION");
            let _ = handshake_server(&stream);
        }
        "verack" => {
            println!(" SE RECIBE VERACK");
        }
        "getheaders" => {
            println!(" SE RECIBE GET HEADERS");
            handle_getheaders(buffer, &mut stream, &headers);
        }
        "getdata" => {
            println!(" SE RECIBE GET DATA");
            handle_getdata(buffer, &mut stream, &blocks);
        }
        _ => {
            println!("Command not found: {}", command);
        }
    }
}
