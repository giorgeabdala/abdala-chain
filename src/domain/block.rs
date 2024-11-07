use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use crate::domain::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block  {
    pub index: usize,
    pub timestamp: String,
    pub proof: u64,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>
}

impl Block {
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!("{}{}{}{}", self.index, self.timestamp, self.proof, self.previous_hash);
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_block_hash() {
        let block = Block {
            index: 0,
            timestamp: "2021-08-01T00:00:00".to_string(),
            proof: 0,
            previous_hash: "0".to_string(),
            transactions: vec![],
        };
        let hash = block.hash();
        assert_eq!(hash.len(), 64);
        assert_eq!(hash.is_empty(), false);
    }

}


