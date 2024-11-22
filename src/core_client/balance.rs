use std::collections::BTreeMap;
use crate::wasm::call::WasmCall;

#[derive(Debug)]
pub struct Pallet {
    pub balance: BTreeMap<String, u64>,
}

impl Pallet {
    pub fn new() -> Self {
        Pallet {
            balance: BTreeMap::new(),
        }
    }

    pub fn balance(&self, address: &str) -> u64 {
        match self.balance.get(address) {
            Some(balance) => *balance,
            None => 0,
        }
    }

    pub fn transfer(&mut self, sender: &str, to: &str, amount: u64) -> Result<(), String> {
        if amount == 0 { return Err("Amount must be greater than 0".to_string()); }
        if self.balance(sender) < amount { return Err("Insufficient balance".to_string()); }

        let sub_result = self.sub_balance(sender, amount);
        if sub_result.is_err() { return Err(sub_result.unwrap_err()); }
        let add_result = self.add_balance(to, amount);
        if add_result.is_err() {
            self.add_balance(sender, amount);
            return Err(add_result.unwrap_err());
        }
        Ok(())
    }

    fn add_balance(&mut self, address: &str, amount: u64) -> Result<(), String> {
        let balance = self.balance(address);
        let new_balance = balance.checked_add(amount);
        if new_balance.is_none() { return Err("Not possible add balance".to_string()); }

        // Create an instance of WasmCall
        let wasm_call = WasmCall::new().map_err(|e| e.to_string())?;
        let mut store = wasm_call;
        let call = store.data().clone();

        // Call the add function of WasmCall
        let result = call.add(&mut store, balance as u32, amount as u32);
        println!("Result from WasmCall add: {}", result);

        self.balance.insert(address.to_string(), new_balance.unwrap());
        Ok(())
    }
    fn sub_balance(&mut self, address: &str, amount: u64) -> Result<(), String> {
        let balance = self.balance(address);
        let new_balance = balance.checked_sub(amount);
        if new_balance.is_none() { return Err("Not possible sub balance".to_string()); }
        self.balance.insert(address.to_string(), new_balance.unwrap());
        Ok(())
    }

    pub fn set_balance(&mut self, address: &str, amount: u64) -> Result<(), String> {
        self.balance.insert(address.to_string(), amount);
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_balance_new() {
        let pallet = Pallet::new();
        assert_eq!(pallet.balance.len(), 0);
    }

    #[test]
    fn test_balance_get() {
        let mut pallet = Pallet::new();
        pallet.balance.insert("Alice".to_string(), 100);
        let balance = pallet.balance("Alice");
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_balance_add() {
        let mut pallet = Pallet::new();
        pallet.add_balance("Alice", 100);
        let balance = pallet.balance("Alice");
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_balance_sub() {
        let mut pallet = Pallet::new();
        pallet.add_balance("Alice", 100);
        pallet.sub_balance("Alice", 50);
        let balance = pallet.balance("Alice");
        assert_eq!(balance, 50);
    }

    #[test]
    fn test_transfer() {
        let mut pallet = Pallet::new();
        pallet.add_balance("Alice", 100);
        pallet.transfer("Alice", "Bob", 50).unwrap();
        let alice_balance = pallet.balance("Alice");
        let bob_balance = pallet.balance("Bob");
        assert_eq!(alice_balance, 50);
        assert_eq!(bob_balance, 50);
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let mut pallet = Pallet::new();
        pallet.set_balance("Alice", 100);
        let result = pallet.transfer("Alice", "Bob", 150);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), "Insufficient balance");
    }

    #[test]
    fn test_transfer_zero_amount() {
        let mut pallet = Pallet::new();
        pallet.set_balance("Alice", 100);
        let result = pallet.transfer("Alice", "Bob", 0);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), "Amount must be greater than 0");
    }

    #[test]
    fn test_set_balance() {
        let mut pallet = Pallet::new();
        pallet.set_balance("Alice", 100);
        let balance = pallet.balance("Alice");
        assert_eq!(balance, 100);
    }



}