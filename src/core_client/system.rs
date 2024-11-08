use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Pallet {
    pub nonce: BTreeMap<String, u64>,
}

impl Pallet {
    pub fn new() -> Self {
        Pallet {
            nonce: BTreeMap::new(),
        }
    }

    pub fn get_nonce(&self, address: &str) -> u64 {
        match self.nonce.get(address) {
            Some(nonce) => *nonce,
            None => 0,
        }
    }

    pub fn increment_nonce(&mut self, address: &str) -> Result<(), String> {
        let nonce = self.get_nonce(address);
        let new_nonce = nonce.checked_add(1);
        if new_nonce.is_none() { return Err("Not possible increment nonce".to_string()); }
        self.nonce.insert(address.to_string(), new_nonce.unwrap());
        Ok(())
    }

    pub fn decrement_nonce(&mut self, address: &str) -> Result<(), String> {
        let nonce = self.get_nonce(address);
        let new_nonce = nonce.checked_sub(1);
        if new_nonce.is_none() { return Err("Not possible decrement nonce".to_string()); }
        self.nonce.insert(address.to_string(), new_nonce.unwrap());
        Ok(())
    }
}

#[cfg(test)]

#[test]
fn test_balance_new() {
    let pallet = Pallet::new();
    assert_eq!(pallet.nonce.len(), 0);
}

#[test]
fn test_increment_nonce() {
    let mut pallet = Pallet::new();
    let address = "0x123";
    let result = pallet.increment_nonce(address);
    assert_eq!(result.is_ok(), true);
    assert_eq!(pallet.get_nonce(address), 1);
}