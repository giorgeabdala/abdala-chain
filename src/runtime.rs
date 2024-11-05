use std::sync::Mutex;
use chrono::Utc;
use digest::Digest;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Block  {
    pub index: usize,
    pub timestamp: String,
    pub proof: u64,
    pub previous_hash: String,
}

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Mutex<Vec<Block>>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().to_rfc3339(),
            proof: 0,
            previous_hash: "0".to_string(),
        };
        chain.push(genesis_block);
        Blockchain {
            chain: Mutex::new(chain),
        }
    }


    pub fn create_block(&self, proof: u64, previous_hash: String) -> Block {
        let mut chain = self.chain.lock().unwrap();
        let block = Block {
            index: chain.len() + 1,
            timestamp: Utc::now().to_rfc3339(),
            proof,
            previous_hash,
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

    pub fn hash(&self, block: &Block) -> String {
        let block_string = serde_json::to_string(&block).unwrap();
        let mut hasher = sha2::Sha256::new();
        hasher.update(block_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn is_chain_valid(&self) -> bool {
        let chain = self.chain.lock().unwrap();
        let mut previous_block = &chain[0];
        let mut block_index = 1;

        while block_index <  chain.len() {
            let block = &chain[block_index];
            if block.previous_hash != self.hash(previous_block) {
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


}

