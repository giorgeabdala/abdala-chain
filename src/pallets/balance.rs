use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Pallet {
    pub balance: BTreeMap<String, u128>,
}

impl Pallet {
    pub fn new() -> Self {
        Pallet {
            balance: BTreeMap::new(),
        }
    }

    pub fn get_balance(&self, address: &str) -> u128 {
        match self.balance.get(address) {
            Some(balance) => *balance,
            None => 0,
        }
    }

    pub fn add_balance(&mut self, address: &str, amount: u128) {
        let balance = self.get_balance(address);
        self.balance.insert(address.to_string(), balance + amount);
    }

    pub fn sub_balance(&mut self, address: &str, amount: u128) {
        let balance = self.get_balance(address);
        self.balance.insert(address.to_string(), balance - amount);
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
        let balance = pallet.get_balance("Alice");
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_balance_add() {
        let mut pallet = Pallet::new();
        pallet.add_balance("Alice", 100);
        let balance = pallet.get_balance("Alice");
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_balance_sub() {
        let mut pallet = Pallet::new();
        pallet.add_balance("Alice", 100);
        pallet.sub_balance("Alice", 50);
        let balance = pallet.get_balance("Alice");
        assert_eq!(balance, 50);
    }
}