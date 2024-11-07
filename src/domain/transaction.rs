use chrono::Utc;
use digest::Digest;
use rocket::serde::Serialize;
use serde::Deserialize;
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub timestamp: String,
    pub sender: String,
    pub to: String,
    pub amount: f64,
    pub message: String,
}


impl Transaction {
    pub fn new(sender: String, to: String, amount: f64, message: String) -> Self {
        let hash = Transaction::hash(&sender, &to, amount, &message);
        Transaction {
            hash,
            timestamp: Utc::now().to_rfc3339(),
            sender,
            to,
            amount,
            message,
        }
    }

    pub fn hash(sender: &str, to: &str, amount: f64, message: &str) -> String {
        let mut hasher = Sha256::new();
        let data = format!("{}{}{}{}", sender, to, amount, message);
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let sender = "Alice";
        let to = "Bob";
        let amount = 10.0;
        let message = "Lohann - Dev Master";
        let hash = Transaction::hash(sender, to, amount, message);
        assert_eq!(hash.len(), 64);
        assert_eq!(hash.is_empty(), false);
    }

    #[test]
    fn test_transaction_new() {
        let sender = "Alice".to_string();
        let to = "Bob".to_string();
        let amount = 10.0;
        let message = "Lohann - Dev Master".to_string();
        let transaction = Transaction::new(sender.clone(), to.clone(), amount, message.clone());
        assert_eq!(transaction.sender, sender);
        assert_eq!(transaction.to, to);
        assert_eq!(transaction.amount, amount);
        assert_eq!(transaction.message, message);
    }
}

