use crate::runtime::{Blockchain};
use rocket::{get, post, routes, serde::json::Json, State};
use rocket::serde::json::serde_json::json;
use tokio::sync::Mutex;
use crate::domain::transaction::Transaction;

#[get("/get_chain")]
async fn get_chain(runtime: &State<Mutex<Blockchain>>) -> Json<serde_json::Value> {
    let runtime = runtime.lock().await;
    Json(json!({
        "chain": runtime.get_chain(),
        "length": runtime.get_chain().len()
    }))
}

#[get("/is_valid")]
async fn is_valid(runtime: &State<Mutex<Blockchain>>) -> Json<serde_json::Value> {
    let runtime = runtime.lock().await;
    let is_valid = runtime.is_chain_valid();
    if is_valid {
        Json(json!({"message": "All good, the blockchain is valid."}))
    } else {
        Json(json!({"message": "The blockchain is not valid."}))
    }
}

#[post("/add_transaction", format = "json", data = "<transaction>")]
async fn add_transaction(runtime: &State<Mutex<Blockchain>>, transaction: Json<Transaction>) -> Json<serde_json::Value> {
    let transaction = transaction.into_inner();
    let transaction = Transaction::new(transaction.sender, transaction.to, transaction.amount, transaction.message);
    let mut runtime = runtime.lock().await;
    runtime.add_transaction(transaction).await;
    Json(json!({"message": "This transaction will be added to the next block"}))
}

#[post("/connect_node", format = "json", data = "<nodes>")]
async fn connect_node(runtime: &State<Mutex<Blockchain>>, nodes: Json<serde_json::Value>) -> Json<serde_json::Value> {
    let nodes = nodes.get("nodes").and_then(|n| n.as_array()).cloned().unwrap_or_default();
    if nodes.is_empty() {
        return Json(json!("Empty"));
    }
    let mut runtime = runtime.lock().await;
    for node in nodes {
        if let Some(node) = node.as_str() {
            runtime.add_node(node.to_string());
        }
    }
    Json(json!({
        "message": "All nodes connected, the blockchain contains the following nodes:",
        "total_nodes": runtime.get_nodes()
    }))
}

#[get("/balance?<address>")]
async fn balance(runtime: &State<Mutex<Blockchain>>, address: &str) -> Json<serde_json::Value> {
    let runtime = runtime.lock().await;
    let balance = runtime.balance(address);
    Json(json!({
        "balance": balance
    }))
}

#[get("/get_nonce?<address>")]
async fn get_nonce(runtime: &State<Mutex<Blockchain>>, address: &str) -> Json<serde_json::Value> {
    let runtime = runtime.lock().await;
    let nonce = runtime.get_nonce(address);
    Json(json!({
        "nonce": nonce
    }))
}

pub async fn start_server() -> Result<(), rocket::Error> {
    let runtime = Blockchain::new();
    rocket::build()
        .manage(Mutex::new(runtime))
        .mount("/", routes![get_chain, is_valid, add_transaction, connect_node, balance, get_nonce])
        .launch()
        .await?;

    Ok(())
}