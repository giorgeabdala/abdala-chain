use std::collections::BTreeMap;
use std::sync::Mutex;
use chrono::Utc;
use digest::Digest;
use serde::{Serialize, Deserialize};
use crate::core_client::balance::Pallet;
use crate::domain::block::Block;

use crate::core_client::system::Pallet as SystemPallet;
use reqwest;
use reqwest::Client;
use serde_json::Value;
use crate::domain::transaction::Transaction;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Mutex<Vec<Block>>,
    transaction_pool: Vec<Transaction>,
    balances: Pallet,
    system: SystemPallet,
    nodes: Vec<String>,

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

        let mut balances = Pallet::new();
        balances.set_balance("Alice", 100).unwrap();

        chain.push(genesis_block);
        Blockchain {
            chain: Mutex::new(chain),
            transaction_pool: vec![],
            balances,
            system: SystemPallet { nonce: Default::default() },
            nodes: vec![],
        }
    }

    fn execute_transaction(&mut self, transaction: Transaction) -> Result<bool, String> {
        let sender = transaction.sender.clone();
        let receiver = transaction.to.clone();
        let amount = transaction.amount;

        let inc_nonce_result = self.system.increment_nonce(&sender);
        if inc_nonce_result.is_err() { return Err(inc_nonce_result.err().unwrap()); }

        if amount <= 0f64 {
            return Err("Amount must be greater than 0".to_string());
        }

        if self.balances.balance(&sender) < amount as u64 {
            return Err("Insufficient balance".to_string());
        }

        let transfer_result = self.balances.transfer(&sender, &receiver, amount as u64);
        if transfer_result.is_err() {
            return Err(transfer_result.err().unwrap());
        }
        Ok(true)
    }

    fn execute_transactions(&mut self, transactions: Vec<Transaction>) -> Result<bool, String> {
        if transactions.is_empty() {
            return Err("No transactions to execute".to_string());
        }
        for transaction in transactions {
            if let Err(e) = self.execute_transaction(transaction.clone()) {
                println!("Error executing transaction: {:?}", e);
                continue;
            }
        }
        Ok(true)
    }

    pub async fn add_transaction(&mut self, transaction: Transaction) {
        let mut transaction = transaction.clone();
        self.consensus().await;
        if transaction.hash.is_empty() {
            transaction = Transaction {
                hash: Transaction::hash(&transaction.sender, &transaction.to, transaction.amount, &transaction.message.clone(), transaction.timestamp),
                timestamp: Some(Utc::now().to_rfc3339()),
                sender: transaction.sender,
                to: transaction.to,
                amount: transaction.amount,
                message: transaction.message,
            } ; }

       match (self.transaction_pool.len()) {
           4 => {
                let previous_block = self.get_previous_block();
                let proof = self.proof_of_work(previous_block.proof);
                let previous_hash = previous_block.hash();

                self.transaction_pool.push(transaction);
                let block = self.create_block(proof, previous_hash);
           }
              _ => {
                self.transaction_pool.push(transaction);
                }
        }
    }

    pub fn create_block(&mut self, proof: u64, previous_hash: String) -> Block {
        let transactions = self.transaction_pool.clone();
        let execute = self.execute_transactions(transactions.clone());

        let mut chain = self.chain.lock().unwrap();

        let block = Block {
            index: chain.len() + 1,
            timestamp: Utc::now().to_rfc3339(),
            proof,
            previous_hash,
            transactions,
        } ;
        chain.push(block.clone());
        self.transaction_pool = vec![];
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

    pub fn add_node(&mut self, address: String) {
        self.nodes.push(address);
    }

    pub fn get_nodes(&self) -> Vec<String> {
        self.nodes.clone()
    }

    async fn consensus(&mut self) {
        let nodes = self.nodes.clone();
        for node in nodes {
            let result = self.replace_chain(node.clone());
            if result.await.is_ok() {
                self.replace_nonce(node.clone()).await;
                self.replace_balance(node.clone()).await;
                println!("Chain replaced");
            }
        }
    }

    async fn replace_nonce(&mut self, node: String) {
        let url = format!("{}/get_all_nonce", node);
        println!("Requesting nonce from: {}", url);

        let client = Client::new();
        let response = client.get(&url).send().await.unwrap();

        if response.status().is_success() {
            let response_json: Value = response.json().await.unwrap();
            let nonce_map = response_json["nonce"].as_object().unwrap();
            self.system.nonce = nonce_map.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap())).collect();
        }
    }

    async fn replace_balance(&mut self, node: String) {
        let url = format!("{}/get_all_balance", node);
        println!("Requesting balance from: {}", url);

        let client = Client::new();
        let response = client.get(&url).send().await.unwrap();

        if response.status().is_success() {
            let response_json: Value = response.json().await.unwrap();
            let balance_map = response_json["balance"].as_object().unwrap();
            self.balances.balance = balance_map.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap())).collect();
        }
    }


    async fn replace_chain(&mut self, node: String) -> Result<bool, Box<dyn std::error::Error>> {
        let mut longest_chain: Option<Vec<Block>> = None;
        let mut max_length = self.chain.lock().unwrap().len();
        let url = format!("{}/get_chain", node);
        println!("Requesting chain from: {}", url);

        let client = Client::new();
        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;
            let length = response_json["length"].as_u64().unwrap() as usize;
            let chain: Vec<Block> = serde_json::from_value(response_json["chain"].clone())?;

            if length > max_length && self.is_chain_valid() {
                max_length = length;
                longest_chain = Some(chain);
            }
        }

        if let Some(chain) = longest_chain {
            let mut current_chain = self.chain.lock().unwrap();
            *current_chain = chain;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn balance(&self, address: &str) -> u64 {
        self.balances.balance(address)
    }

    pub fn get_nonce(&self, address: &str) -> u64 {
        self.system.get_nonce(address)
        }

    pub fn get_all_nonce(&self) -> BTreeMap<String, u64> {
        self.system.nonce.clone()
    }

    pub fn get_all_balance(&self) -> BTreeMap<String, u64> {
        self.balances.balance.clone()
    }





    pub fn set_balance(&mut self, address: &str, amount: u64) -> Result<(), String> {
        self.balances.set_balance(address, amount)
    }

}


#[cfg(test)]

//gere os testes para o módulo runtime
mod tests {
    use digest::typenum::assert_type;
    use super::*;

    #[test]
    fn test_blockchain_new() {
        let blockchain = Blockchain::new();
        let chain = blockchain.chain.lock().unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(blockchain.transaction_pool.len(), 0);
        assert_eq!(blockchain.balances.balance("Alice"), 100);
        assert_eq!(blockchain.system.get_nonce("Alice"), 0);

    }

    #[test]
    fn test_create_block() {
        let mut blockchain = Blockchain::new();
        let previous_block = blockchain.get_previous_block();
        let proof = blockchain.proof_of_work(previous_block.proof);
        let previous_hash = previous_block.hash();

        let transaction = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50f64,
            message: "".to_string(),
        };

        let block = blockchain.create_block(proof, previous_hash.clone());

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
        blockchain.balances.set_balance("Alice", 100);
        blockchain.balances.set_balance("Bob", 100);
        let transaction = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50f64,
            message: "".to_string(),
        };
        let result = blockchain.execute_transaction(transaction);
        assert_eq!(result.is_ok(), true);
        assert_eq!(blockchain.balances.balance("Alice"), 50);
        assert_eq!(blockchain.balances.balance("Bob"), 150);
        assert_eq!(blockchain.system.get_nonce("Alice"), 1);
    }

    #[test]
    fn test_execute_transactions() {
        let mut blockchain = Blockchain::new();
        blockchain.balances.set_balance("Alice", 100);
        blockchain.balances.set_balance("Bob", 100);
        let transaction1 = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50f64,
            message: "".to_string(),
        };
        let transaction2 = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: "Bob".to_string(),
            to: "Alice".to_string(),
            amount: 25f64,
            message: "".to_string(),
        };
        let transaction3 = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: "Bob".to_string(),
            to: "Alice".to_string(),
            amount: 25f64,
            message: "".to_string(),
        };
        let transactions = vec![transaction1, transaction2, transaction3];
        let result = blockchain.execute_transactions(transactions);
        assert_eq!(result.is_ok(), true);
        assert_eq!(blockchain.balances.balance("Alice"), 100);
        assert_eq!(blockchain.balances.balance("Bob"), 100);
        assert_eq!(blockchain.system.get_nonce("Alice"), 1);
        assert_eq!(blockchain.system.get_nonce("Bob"), 2);
    }

    #[tokio::test]
    async fn test_add_transaction() {
        let mut blockchain = Blockchain::new();
        let sender = "Alice".to_string();
        let to = "Bob".to_string();
        let amount = 50f64;


        let transaction = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: sender.clone(),
            to: to.clone(),
            amount: amount,
            message: "".to_string(),
        };

        for _ in 0..4 {
            blockchain.add_transaction(transaction.clone()).await;
        }

        let transaction_pool = blockchain.transaction_pool.clone();
        assert_eq!(transaction_pool.len(), 4);
        blockchain.add_transaction(transaction.clone()).await;
        let transaction_pool = blockchain.transaction_pool.clone();
        assert_eq!(transaction_pool.len(), 0);
        blockchain.add_transaction(transaction.clone()).await;
        let transaction_pool = blockchain.transaction_pool.clone();
        assert_eq!(transaction_pool[0].sender, sender);
        assert_eq!(transaction_pool[0].to, to);
        assert_eq!(transaction_pool[0].amount, amount);
        assert_eq!(transaction_pool[0].hash.is_empty(), false);
    }

    #[test]
    fn test_execute_transaction_fail()
    {
        let mut blockchain = Blockchain::new();
        blockchain.balances.set_balance("Alice", 100);
        blockchain.balances.set_balance("Bob", 100);
        let transaction = Transaction {
            hash: "".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            sender: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 150f64,
            message: "".to_string(),
        };
        let result = blockchain.execute_transaction(transaction);
        assert_eq!(result.is_err(), true);
        assert_eq!(blockchain.balances.balance("Alice"), 100);
        assert_eq!(blockchain.balances.balance("Bob"), 100);
        assert_eq!(blockchain.system.get_nonce("Alice"), 1);
        assert_eq!(blockchain.system.get_nonce("Bob"), 0);
    }

    #[test]
    fn test_balance() {
        let mut blockchain = Blockchain::new();
        let balance = blockchain.balance("Alice");
        assert_eq!(balance, 100);

        blockchain.balances.set_balance("Alice", 200);
        let balance = blockchain.balance("Alice");
        assert_eq!(balance, 200);
    }

    #[test]
    fn test_get_nonce() {
        let mut blockchain = Blockchain::new();
        let nonce = blockchain.get_nonce("Alice");
        assert_eq!(nonce, 0);

        blockchain.system.increment_nonce("Alice").unwrap();
        let nonce = blockchain.get_nonce("Alice");
        assert_eq!(nonce, 1);
    }

    #[test]
    fn test_get_all_nonce() {
        let mut blockchain = Blockchain::new();
        let nonce = blockchain.get_all_nonce();
        assert_eq!(nonce.len(), 0);

        blockchain.system.increment_nonce("Alice").unwrap();
        let nonce = blockchain.get_all_nonce();
        assert_eq!(nonce.len(), 1);
        assert_eq!(nonce["Alice"], 1);
    }



    #[test]
    fn test_set_balance() {
        let mut blockchain = Blockchain::new();
        let result = blockchain.set_balance("Alice", 100);
        assert_eq!(result.is_ok(), true);
        let balance = blockchain.balance("Alice");
        assert_eq!(balance, 100);
    }


    //###### Testes above only work in network. ##########
    /*#[tokio::test]
    async fn test_replace_chain() {
        let mut blockchain = Blockchain::new();
        let node = "http://localhost:8088";
        blockchain.add_node(node.to_string());
        let node = blockchain.get_nodes()[0].clone();
        println!("Nodes: {}", node);
        let result = blockchain.replace_chain(node.to_string()).await;
        assert_eq!(result.is_ok(), true);
    }

    #[tokio::test]
    async fn test_replace_nonce() {
        let mut blockchain = Blockchain::new();
        let node = "http://localhost:8088";
        blockchain.add_node(node.to_string());
        let node = blockchain.get_nodes()[0].clone();
        println!("Nodes: {}", node);
        blockchain.system.increment_nonce("Alice").unwrap();
        blockchain.replace_nonce(node.to_string()).await;
        let nonce = blockchain.system.get_nonce("Alice");
        assert_eq!(nonce, 0);
    }

    #[tokio::test]
    async fn test_replace_balance() {
        let mut blockchain = Blockchain::new();
        let node = "http://localhost:8088";
        blockchain.add_node(node.to_string());
        let node = blockchain.get_nodes()[0].clone();
        println!("Nodes: {}", node);
        blockchain.balances.set_balance("Alice", 200).expect("TODO: set balance error");
        blockchain.replace_balance(node.to_string()).await;
        let balance = blockchain.balance("Alice");
        assert_eq!(balance, 100);
    }*/

}
