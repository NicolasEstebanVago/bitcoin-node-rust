use std::io::Error;

use crate::{
    components::{block::Block, block_header::BlockHeader},
    helpers::auxiliar_functions::u8_to_hex_string,
};

use super::logger_impl::Logger;
pub fn log_block_header(
    logger_result: Option<&Logger>,
    block_header: &BlockHeader,
) -> Result<String, Error> {
    let logger = match logger_result {
        Some(logger) => logger,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to get logger",
            ));
        }
    };

    let mut block_string = String::new();

    if let Some(_logger) = logger_result {
        block_string.push_str(&format!(
            "| ( {}, {:?}, {:?}, {},  {}, {} ), (",
            block_header.version,
            block_header.prev_block_hash,
            block_header.merkle_root,
            block_header.timestamp,
            block_header.bits,
            block_header.nonce
        ));
    }

    logger.log(&block_string)?;

    Ok("Successfully logged".to_string())
}

pub fn log_block(logger_result: Option<&Logger>, block: &Block) -> Result<String, Error> {
    let mut block_string = String::new();

    if let Some(logger) = logger_result {
        block_string.push_str(&format!(
            "| ( {}, {:?}, {:?}, {},  {}, {} ), (",
            block.header.version,
            block.header.prev_block_hash,
            block.header.merkle_root,
            block.header.timestamp,
            block.header.bits,
            block.header.nonce
        ));

        for transaction in &block.txns {
            block_string.push_str(&format!(
                "{{ hash : {},  version : {}, tx_in_count : {}, inputs : {:?}, tx_out_count : {}, outputs : {:?}, lock_time : {}, txid : {:?} }}",
                u8_to_hex_string(&transaction.hash),
                transaction.version,
                transaction.tx_in_count,
                transaction.inputs,
                transaction.tx_out_count,
                transaction.outputs,
                transaction.lock_time,
                transaction.txid)
            );
        }
        block_string.push_str(") |");
        logger.log(&block_string)?;
        Ok("Successfully logged".to_string())
    } else {
        Err(Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to get logger",
        ))
    }
}
