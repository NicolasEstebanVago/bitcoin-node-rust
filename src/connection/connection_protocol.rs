use crate::{
    configuration::config_helper::get_configuration,
    testnet_protocol::messages::message_builders::{
        build_verack_header_message, build_version_message,
    },
};

use std::{
    io::{Error, ErrorKind},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

use std::io::{Read, Write};
use std::vec;
use std::vec::IntoIter;

pub fn set_tcp_stream_vec(nodes: Vec<String>) -> Result<Vec<TcpStream>, Error> {
    let mut tcp_stream_vec: Vec<TcpStream> = vec![];
    println!("nodes {:?}", nodes);
    for i in &nodes {
        let mut addrs = i.to_socket_addrs()?;
        if let Some(addr) = addrs.next() {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
                Ok(socket_1) => {
                    println!(" socket {:?} ", socket_1);
                    handshake(&socket_1)?;
                    tcp_stream_vec.push(socket_1);
                }
                Err(_e) => {
                    continue;
                }
            }
        }
    }

    Ok(tcp_stream_vec)
}

pub fn fetch_nodes_config() -> Result<Vec<String>, Error> {
    let mut config = get_configuration()
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get the configuration"))?;

    let dns_protocol = config
        .get_value_from_key("protocol_version".to_owned())
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get the protocol version"))?;

    let dns_direction = config
        .get_value_from_key("direction".to_owned())
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get direction"))?;

    let addrs_iter: IntoIter<SocketAddr> = format!("{}:{}", dns_direction, dns_protocol)
        .to_socket_addrs()
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to convert to socket addresses"))?;

    let custom_ip = config
        .get_value_from_key("custom_ip".to_owned())
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get cusotm_ip"))?;

    let nodes: Vec<String> = if !custom_ip.is_empty() {
        format!("{}:{}", custom_ip, dns_protocol)
            .to_socket_addrs()?
            .chain(addrs_iter)
            .map(|addr| addr.to_string())
            .collect()
    } else {
        addrs_iter.map(|addr| addr.to_string()).collect()
    };

    println!("Peers discovered ---> {:?}", nodes.len());

    Ok(nodes)
}

pub fn handshake_server(mut socket: &TcpStream) -> Result<(), Error> {
    let version_msg = build_version_message()?;
    socket.write_all(&version_msg)?;

    let verack_msg = build_verack_header_message();
    socket.write_all(&verack_msg)?;

    Ok(())
}

pub fn handshake(mut socket: &TcpStream) -> Result<(), Error> {
    // Build and send version message
    let version_msg = build_version_message()?;
    socket.write_all(&version_msg)?;

    // lectura del version del servidor
    let mut response_buffer = [0; 1024];
    let _ = socket.read(&mut response_buffer)?;

    let header_verack_msg = build_verack_header_message();
    socket.write_all(&header_verack_msg)?;

    Ok(())
}
