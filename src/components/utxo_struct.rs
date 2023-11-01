use secp256k1::PublicKey;
#[derive(Debug, Clone)]
pub struct Utxo {
    pub txid: String,      // transaction id
    pub index: u32,        // output index transaction
    pub value: u64,        // ouptut value
    pub pubkey: PublicKey, // public key of the user
}
