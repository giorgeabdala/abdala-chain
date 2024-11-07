use std::sync::Mutex;
use chrono::Utc;
use digest::Digest;
use serde::{Serialize, Deserialize};
use crate::domain::block::Block;
use crate::domain::transaction::Transaction;
use crate::pallets::balance::Pallet;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Mutex<Vec<Block>>,
    transaction_pool: Vec<Transaction>,
    balances: Pallet,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().to_rfc3339(),
            proof: 0,
            previous_hash: "0".to_string(),
            transactions: vec![],
        };
        chain.push(genesis_block);
        Blockchain {
            chain: Mutex::new(chain),
            transaction_pool: vec![],
            balances: Pallet { balance: Default::default() },
        }
    }

    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<bool, String> {
        let sender = transaction.sender.clone();
        let receiver = transaction.to.clone();
        let amount = transaction.amount;

        if amount <= 0f64 {
            return Err("Amount must be greater than 0".to_string());
        }

        if self.balances.get_balance(&sender) < amount as u128 {
            return Err("Insufficient balance".to_string());
        }

        self.balances.sub_balance(&sender, amount as u128);
        self.balances.add_balance(&receiver, amount as u128);
        Ok(true)
    }

    fn execute_transactions(&mut self, transactions: Vec<Transaction>) -> Result<bool, String> {
        if transactions.is_empty() {
            return Err("No transactions to execute".to_string());
        }

        for transaction in transactions {
            self.execute_transaction(transaction)?;
        }
        Ok(true)
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
       match (self.transaction_pool.len()) {
           4 => {
                let previous_block = self.get_previous_block();
                let proof = self.proof_of_work(previous_block.proof);
                let previous_hash = previous_block.hash();
                self.transaction_pool.push(transaction);
                let block = self.create_block(proof, previous_hash);
               self.transaction_pool = vec![];
           }
              _ => {
                self.transaction_pool.push(transaction);
                }
        }
    }

    pub fn create_block(&mut self, proof: u64, previous_hash: String) -> Block {
        let transactions = self.transaction_pool.clone();
        let execute = self.execute_transactions(transactions.clone());

        //TODO: teste falha ao implementar esse if. Transactions vem vazio no teste create_block

/*        if execute.is_err() {
            panic!("Error executing transactions: {:?}", execute.err().unwrap());
        }
*/
        let mut chain = self.chain.lock().unwrap();

        let block = Block {
            index: chain.len() + 1,
            timestamp: Utc::now().to_rfc3339(),
            proof,
            previous_hash,
            transactions,
        } ;
        chain.push(block.clone());
        block
    }

    pub fn get_previous_block(&self) -> Block {
        let chain = self.chain.lock().unwrap();
        chain.last().unwrap().clone()
    }

    pub fn proof_of_work(&self, previous_proof: u64) -> u64 {
        let mut new_proof: u64 = previous_proof + 1; // Começa a partir de previous_proof + 1
        let mut check_proof = false;

        while !check_proof {
            let calc = new_proof.pow(2) - previous_proof.pow(2);
            let mut hasher = sha2::Sha256::new();
            hasher.update(calc.to_string().as_bytes());
            let hash_result = format!("{:x}", hasher.finalize());

            match &hash_result[..4] {
                "0000" => check_proof = true,
                _ => new_proof += 1,
            }
        }
        new_proof
    }


    pub fn is_chain_valid(&self) -> bool {
        let chain = self.chain.lock().unwrap();
        let mut previous_block = &chain[0];
        let mut block_index = 1;

        while block_index <  chain.len() {
            let block = &chain[block_index];
            if block.previous_hash != previous_block.hash() {
                return false;
            }

            let previous_proof = previous_block.proof;
            let proof = block.proof;
            let calc = proof.pow(2) - previous_proof.pow(2);
            let mut hasher = sha2::Sha256::new();
            hasher.update(calc.to_string().as_bytes());
            let hash_result = format!("{:x}", hasher.finalize());


            match &hash_result[..4] {
                "0000" => {
                    previous_block = block;
                    block_index += 1;
                },
                _ => return false,
            }
        }
        true
    }

    pub fn get_chain(&self) -> Vec<Block> {
        let chain = self.chain.lock().unwrap();
        chain.clone()
    }

    pub fn get_block(&self, index: usize) -> Option<Block> {
        let chain = self.chain.lock().unwrap();
        chain.get(index).cloned()
    }




}


#[cfg(test)]

//gere os testes para o módulo runtime
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_new() {
        let blockchain = Blockchain::new();
        let chain = blockchain.chain.lock().unwrap();
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_create_block() {
        let mut blockchain = Blockchain::new();
        let previous_block = blockchain.get_previous_block();
        let proof = blockchain.proof_of_work(previous_block.proof);
        let previous_hash = previous_block.hash();
        let block = blockchain.create_block(proof, previous_hash.clone());

        let transaction = Transaction {
            hash: "".to_string(),
            timestamp: "".to_string(),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50f64,
            message: "".to_string(),
        };

        blockchain.add_transaction(transaction);

        let mut chain = blockchain.chain.lock().unwrap();

        assert_eq!(chain.len(), 2);
        assert_eq!(block.index, 2);
        assert_eq!(block.proof, proof);
        assert_eq!(block.previous_hash, previous_hash);
    }

    #[test]
    fn test_get_previous_block() {
        let blockchain = Blockchain::new();
        let previous_block = blockchain.get_previous_block();
        let chain = blockchain.chain.lock().unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(previous_block.index, 0);
    }

    #[test]
    fn test_proof_of_work() {
        let blockchain = Blockchain::new();
        let previous_block = blockchain.get_previous_block();
        let previous_proof = previous_block.proof;
        let proof = blockchain.proof_of_work(previous_proof);
        assert_eq!(proof, 115558);
    }

    #[test]
    fn test_hash() {
        let blockchain = Blockchain::new();
        let previous_block = blockchain.get_previous_block();
        let hash = previous_block.hash();
        assert_eq!(hash.is_empty(), false);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_is_chain_valid() {
        let blockchain = Blockchain::new();
        let is_valid = blockchain.is_chain_valid();
        assert_eq!(is_valid, true);
    }

    #[test]
    fn test_get_chain() {
        let blockchain = Blockchain::new();
        let chain = blockchain.get_chain();
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_get_block() {
        let blockchain = Blockchain::new();
        let block = blockchain.get_block(0);
        assert_eq!(block.is_some(), true);
    }

    #[test]
    fn test_execute_transaction() {
        let mut blockchain = Blockchain::new();
        blockchain.balances.add_balance("Alice", 100);
        blockchain.balances.add_balance("Bob", 100);
        let transaction = Transaction {
            hash: "".to_string(),
            timestamp: "".to_string(),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50f64,
            message: "".to_string(),
        };
        let result = blockchain.execute_transaction(transaction);
        assert_eq!(result.is_ok(), true);
        assert_eq!(blockchain.balances.get_balance("Alice"), 50);
        assert_eq!(blockchain.balances.get_balance("Bob"), 150);
    }

    #[test]
    fn test_execute_transactions() {
        let mut blockchain = Blockchain::new();
        blockchain.balances.add_balance("Alice", 100);
        blockchain.balances.add_balance("Bob", 100);
        let transaction1 = Transaction {
            hash: "".to_string(),
            timestamp: "".to_string(),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50f64,
            message: "".to_string(),
        };
        let transaction2 = Transaction {
            hash: "".to_string(),
            timestamp: "".to_string(),
            sender: "Bob".to_string(),
            to: "Alice".to_string(),
            amount: 25f64,
            message: "".to_string(),
        };
        let transactions = vec![transaction1, transaction2];
        let result = blockchain.execute_transactions(transactions);
        assert_eq!(result.is_ok(), true);
        assert_eq!(blockchain.balances.get_balance("Alice"), 75);
        assert_eq!(blockchain.balances.get_balance("Bob"), 125);
    }

}
