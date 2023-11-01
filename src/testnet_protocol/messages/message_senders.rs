use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use crate::testnet_protocol::messages::message_parsers::parse_message;

pub fn empty_socket(socket: &mut TcpStream) {
    let mut response_buffer = [0; 5000];
    let _ = socket.set_read_timeout(Some(Duration::from_secs(1)));

    match socket.read(&mut response_buffer) {
        Ok(mut response_size) => {
            while response_size == 5000 {
                response_buffer = [0; 5000];
                response_size = socket.read(&mut response_buffer).unwrap();
            }
        }
        Err(_) => {
            println!("Error al vaciar socket");
        }
    }
}

pub fn write_and_read_get_headers_message(
    get_headers: &[u8],
    socket: &mut TcpStream,
) -> Result<Vec<u8>, Error> {
    socket.write_all(get_headers)?;

    let mut response_buffer = [0; 162027];

    let response_size = socket.read(&mut response_buffer)?;

    println!("Sexta lectura, luego de enviar mensaje get headers");
    println!("Tamaño de la sexta lectura ---> {}", response_size);
    parse_message(&response_buffer[..24], None)?;

    if response_size < 162027 {
        empty_socket(socket);
    }

    println!(
        "La longitud del buffer que se esta devolviendo es de ---> {}",
        response_buffer[24..].len()
    );

    Ok(response_buffer[24..].to_vec())
}

pub fn write_and_read_get_data_message(
    get_data_message: &[u8],
    mut socket: &TcpStream,
) -> Result<(u32, Vec<u8>), String> {
    socket.write_all(get_data_message).unwrap();

    let mut response_buffer = [0; 100000];
    let mut response_size = socket.read(&mut response_buffer).unwrap();

    println!(
        " ---------- Tamaño de lectura post enviar get data ---------- {} ",
        response_size
    );

    let (mut command, mut payload) = parse_message(&response_buffer, None).unwrap();

    let mut errores = 0;

    while command != *"block" {
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
    }

    Ok((payload, response_buffer[..response_size].to_vec()))
}
