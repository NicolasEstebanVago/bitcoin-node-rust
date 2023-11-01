use super::utxo_struct::Utxo;

#[derive(Debug, Clone)]
pub struct UTXOSet {
    pub utxos: Vec<Utxo>,
}

#[allow(dead_code)]
impl UTXOSet {
    // Constructor for UTXOSet
    pub fn new() -> Self {
        UTXOSet { utxos: Vec::new() }
    }

    // Function to add a UTXO to the set
    pub fn add_utxo(&mut self, utxo: Utxo) {
        self.utxos.push(utxo);
    }

    // Function to remove a UTXO from the set
    pub fn remove_utxo(&mut self, txid: &str, index: u32) {
        self.utxos.retain(|u| !(u.txid == txid && u.index == index));
    }

    // Function to search for a UTXO by its ID and index
    pub fn find_utxo(&self, txid: &str, index: u32) -> Option<&Utxo> {
        self.utxos
            .iter()
            .find(|u| u.txid == txid && u.index == index)
    }
}
