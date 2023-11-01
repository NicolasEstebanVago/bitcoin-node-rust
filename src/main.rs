use crate::configuration::config_helper::get_configuration;

use connection::connection_modes::{client_mode, server_mode};
use std::io::{Error, ErrorKind};

mod configuration {
    pub mod config_helper;
    pub mod configuration_loader;
}

mod testnet_protocol;

mod logger {
    pub mod log_printer;
    pub mod logger_impl;
}

mod components {
    pub mod block;
    pub mod block_header;
    pub mod transaction;
    pub mod user;
    pub mod utxo_set;
    pub mod utxo_struct;
    pub mod wallet;
}

mod helpers {
    pub mod auxiliar_functions;
    pub mod persistance;
}

mod merkle_tree {
    pub mod merkle_tree_calculator;
}

mod connection {
    pub mod connection_modes;
    pub mod connection_protocol;
}

mod interface {
    pub mod interfaz_grafica;
}

fn main() -> Result<(), Error> {
    let mut config = get_configuration()
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get the configuration"))?;

    let mode = config
        .get_value_from_key("mode".to_owned())
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to get the mode"))?;

    match mode.as_str() {
        "client" => client_mode(),
        "server" => server_mode()?,
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "Invalid mode specified in the configuration file",
            ));
        }
    }

    Ok(())
}
