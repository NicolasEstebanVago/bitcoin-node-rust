use bitcoin_hashes::{sha256d, Hash};

use crate::configuration::config_helper::get_configuration;
use crate::configuration::configuration_loader::ConfigurationError;

use std::io::Error;

use std::io::ErrorKind::{self, InvalidData};

const GET_DATA_MSG: &[u8; 12] = b"getdata\0\0\0\0\0";
const GET_HEADERS_MSG: &[u8; 12] = b"getheaders\0\0";

/// Implementation to use the enum configuration Error
impl From<ConfigurationError> for std::io::Error {
    fn from(err: ConfigurationError) -> Self {
        match err {
            ConfigurationError::ValueNotExist => Error::new(ErrorKind::NotFound, "Value not found"),
            ConfigurationError::ReadFileFail => {
                Error::new(ErrorKind::Interrupted, "Value not found")
            }
            ConfigurationError::ReadLineFail => {
                Error::new(ErrorKind::Interrupted, "Value not found")
            }
            ConfigurationError::FormatFileLineFail => {
                Error::new(ErrorKind::InvalidData, "Value not found")
            }
        }
    }
}

/// Builds the version message to send to the server
pub fn build_version_message() -> Result<Vec<u8>, Error> {
    let mut config = get_configuration()?;
    let version = config.get_value_from_key("version".to_owned())?;
    let addr_recv_ipv4 = config.get_value_from_key("addr_recv_ipv4".to_owned())?;
    let addr_trans_ipv4 = config.get_value_from_key("addr_trans_ipv4".to_owned())?;

    let mut payload: Vec<u8> = Vec::new();

    let version: i32 = match version.parse() {
        Ok(parsed_value) => parsed_value,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to parse version as i32",
            ));
        }
    };
    payload.extend_from_slice(&version.to_le_bytes());

    // services, 8 bytes uint64_t
    let services: u64 = 1;
    payload.extend_from_slice(&services.to_le_bytes());

    // timestamp, 8 bytes int64_t
    let timestamp: i64 = chrono::Utc::now().timestamp();
    payload.extend_from_slice(&timestamp.to_le_bytes());

    // addr_recv services, 8 bytes uint64_t
    let addr_recv_serv: u64 = 0x01;
    payload.extend_from_slice(&addr_recv_serv.to_be_bytes());

    // addr_recv IP address, 16 bytes, char[16], Big Endian
    let addr_recv_ipv4 = addr_recv_ipv4
        .parse::<std::net::Ipv4Addr>()
        .map_err(|e| Error::new(InvalidData, e))?;
    let addr_recv_ip = addr_recv_ipv4.to_ipv6_mapped();
    payload.extend_from_slice(&addr_recv_ip.octets());

    // addr_recv_port, 2 bytes
    let port: u16 = 18333;
    payload.extend_from_slice(&port.to_be_bytes());

    // addr_trans_services, 8 bytes
    let addr_trans_services: u64 = 0x01;
    payload.extend_from_slice(&addr_trans_services.to_be_bytes());

    // addr_trans IP address, 16 bytes, char[16], Big Endian
    let addr_trans_ipv4 = addr_trans_ipv4
        .parse::<std::net::Ipv4Addr>()
        .map_err(|e| Error::new(InvalidData, e))?;
    let addr_trans_ip = addr_trans_ipv4.to_ipv6_mapped();
    payload.extend_from_slice(&addr_trans_ip.octets());

    // addr_trans_port
    let port_trans: u16 = 18333;
    payload.extend_from_slice(&port_trans.to_be_bytes());

    // nonce
    let nonce: u64 = 0;
    payload.extend_from_slice(&nonce.to_le_bytes());

    // user_agent_bytes, Varies
    let user_agent_bytes: u8 = 0x00; // cambio de u32
    payload.extend_from_slice(&user_agent_bytes.to_be_bytes());

    // start_height, 4 bytes
    let start_height: i32 = 788428;
    payload.extend_from_slice(&start_height.to_le_bytes());

    let relay: u8 = 0o1;
    payload.extend_from_slice(&relay.to_le_bytes());

    // Construct the header
    let mut header = build_header_message(payload.clone(), b"version\0\0\0\0\0");

    header.extend_from_slice(&payload);

    Ok(header)
}

/// Builds the inv message
#[allow(dead_code)]
pub fn build_inv_message(transaction_hash: Vec<u8>) -> Result<Vec<u8>, Error> {
    // payload
    let mut payload: Vec<u8> = Vec::new();

    // Number of inventory entries
    let count: u8 = 1;
    payload.extend_from_slice(&count.to_le_bytes());

    // Inventory inv_vec (36 bytes x count)

    // type of the hash (1 = transaction)
    let type_hash: u32 = 1;
    payload.extend_from_slice(&type_hash.to_le_bytes());

    // hash of the transaction

    println!(
        "longitud del transaction hash enviado ---> {}",
        transaction_hash.len()
    );

    payload.extend_from_slice(&transaction_hash);

    let mut header = build_header_message(payload.clone(), b"inv\0\0\0\0\0\0\0\0\0");

    // Combine header and payload
    header.extend_from_slice(&payload);

    Ok(header)
}

/// Build the tx message
pub fn build_tx_message(transaction_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut payload: Vec<u8> = Vec::new();

    payload.extend_from_slice(&transaction_bytes);

    let mut header = build_header_message(payload.clone(), b"tx\0\0\0\0\0\0\0\0\0\0");

    // Combine header and payload
    header.extend_from_slice(&payload);

    Ok(header)
}

/// Constructs the header with all its properties
pub fn build_verack_header_message() -> Vec<u8> {
    let mut header: Vec<u8> = Vec::new();

    let magic: u32 = 0x0709110b;
    header.extend_from_slice(&magic.to_le_bytes());
    header.extend_from_slice(b"verack\0\0\0\0\0\0");

    let payload_length: u32 = 0;
    header.extend_from_slice(&payload_length.to_le_bytes());

    let payload: Vec<u8> = Vec::new();
    let checksum = bitcoin_hashes::sha256d::Hash::hash(&payload);
    header.extend_from_slice(&checksum[0..4]);
    header
}

/// Build the get headers message with all its fields
pub fn build_get_headers_message(block_locator_hash: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut payload: Vec<u8> = Vec::new();

    let version: u32 = 70015;

    payload.extend_from_slice(&version.to_le_bytes());

    let num_block_locator_hashes: u8 = 1;
    payload.extend_from_slice(&num_block_locator_hashes.to_le_bytes());

    payload.extend_from_slice(&block_locator_hash);

    let hash_stop = vec![0; 32];
    payload.extend_from_slice(&hash_stop);

    // Construct the header
    let mut header = build_header_message(payload.clone(), GET_HEADERS_MSG);

    header.extend_from_slice(&payload);

    Ok(header)
}

/// Builds the get data message of the block
pub fn build_get_data_message(prev_block_hash: &[u8]) -> Result<Vec<u8>, Error> {
    let mut payload: Vec<u8> = Vec::new();

    let num_inv_vect: u8 = 1;
    payload.extend_from_slice(&num_inv_vect.to_le_bytes());

    let object_type: u32 = 2;
    payload.extend_from_slice(&object_type.to_le_bytes());

    let block_hash: Vec<u8> = prev_block_hash.to_owned();
    payload.extend_from_slice(&block_hash);

    let mut header = build_header_message(payload.clone(), GET_DATA_MSG);

    header.extend_from_slice(&payload);

    Ok(header)
}

/// Builds the headers message
pub fn build_header_message(payload: Vec<u8>, command: &[u8; 12]) -> Vec<u8> {
    let mut header: Vec<u8> = Vec::new();
    let magic: u32 = 0x0709110b;
    header.extend_from_slice(&magic.to_le_bytes());
    header.extend_from_slice(command);

    let payload_length: u32 = payload.len() as u32;
    header.extend_from_slice(&payload_length.to_le_bytes());

    let checksum = sha256d::Hash::hash(&payload);

    header.extend_from_slice(&checksum[0..4]);

    header
}
