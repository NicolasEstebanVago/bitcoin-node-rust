pub mod messages {
    pub mod message_builders;
    pub mod message_parsers;
    pub mod message_senders;
}

pub mod client_handlers {
    pub mod handle_getdata;
    pub mod handle_getheaders;
    pub mod tests;
}

pub mod block_download;
pub mod broadcasting;
pub mod header_download;
